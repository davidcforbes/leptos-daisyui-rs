use super::types::{
    FlatNode, KeyNav, TreeLoader, TreeNode, build_flat, child_indices, flat_children, handle_key,
    index_of_key, insert_children, row_key, should_spawn_load,
};
use crate::merge_classes;
use leptos::{ev::KeyboardEvent, html, prelude::*};
use std::hash::{DefaultHasher, Hash, Hasher};
use std::sync::atomic::{AtomicU64, Ordering};
use wasm_bindgen::JsCast;
use web_sys::{ScrollIntoViewOptions, ScrollLogicalPosition};

/// Per-instance sequence so each `Tree` gets unique treeitem DOM ids for
/// `aria-activedescendant` wiring (WAI-ARIA tree pattern, mirroring
/// `ResultList`'s listbox wiring).
static TREE_SEQ: AtomicU64 = AtomicU64::new(0);

/// A stable DOM id for a node, derived from its (unchanging) key rather than
/// its flat index — so `aria-activedescendant` / scroll-into-view keep working
/// across the index shifts caused by lazy insertion and collapse.
fn key_dom_id(instance: u64, key: &str) -> String {
    let mut hasher = DefaultHasher::new();
    key.hash(&mut hasher);
    format!("ld-tree-{instance}-k{}", hasher.finish())
}

/// Shared state threaded through the recursive row renderer. All fields are
/// `Copy`/cheap-`Clone` (signals are `Copy`, [`TreeLoader`] is `Arc`-backed and
/// `Send + Sync`), so `Ctx` lives happily in Leptos's threadsafe reactive graph.
#[derive(Clone)]
struct Ctx {
    instance: u64,
    store: RwSignal<Vec<FlatNode>>,
    selected: RwSignal<Option<String>>,
    hover: RwSignal<Option<String>>,
    loader: Option<TreeLoader>,
    on_activate: Option<Callback<String>>,
    on_toggle: Option<Callback<String>>,
    on_selection_change: Option<Callback<Option<String>>>,
}

/// Expand or collapse the branch identified by `key`, lazily loading its
/// children on first expand via the async [`TreeLoader`]. All mutations locate
/// the node **by key** (never a captured index) so they stay correct across the
/// index shifts caused by earlier insertions/collapses.
fn toggle_by_key(ctx: &Ctx, key: String) {
    let store = ctx.store;
    let Some(idx) = store.with_untracked(|n| index_of_key(n, &key)) else {
        return;
    };
    let node = store.with_untracked(|n| n[idx].clone());
    if !node.is_branch {
        return;
    }

    if node.expanded {
        store.update(|n| {
            if let Some(i) = index_of_key(n, &key) {
                n[i].expanded = false;
            }
        });
        if let Some(cb) = ctx.on_toggle {
            cb.run(key);
        }
        return;
    }

    // Expanding.
    if node.children_loaded {
        store.update(|n| {
            if let Some(i) = index_of_key(n, &key) {
                n[i].expanded = true;
            }
        });
        if let Some(cb) = ctx.on_toggle {
            cb.run(key);
        }
        return;
    }

    if !should_spawn_load(&node) {
        // Already loading — e.g. this branch was collapsed while its first
        // load was still in flight, and is now being re-expanded before that
        // load resolved. Just restore `expanded` and let the in-flight load
        // complete on its own; spawning a second load here is the double-load
        // race (both completions would splice children, duplicating rows).
        store.update(|n| {
            if let Some(i) = index_of_key(n, &key) {
                n[i].expanded = true;
            }
        });
        if let Some(cb) = ctx.on_toggle {
            cb.run(key);
        }
        return;
    }

    match ctx.loader.clone() {
        Some(loader) => {
            // Show the loading state immediately, then fetch children.
            store.update(|n| {
                if let Some(i) = index_of_key(n, &key) {
                    n[i].expanded = true;
                    n[i].loading = true;
                }
            });
            spawn_load_children(store, loader, key, ctx.on_toggle);
        }
        None => {
            // No loader configured: treat the branch as empty-but-loaded.
            store.update(|n| {
                if let Some(i) = index_of_key(n, &key) {
                    n[i].expanded = true;
                    n[i].children_loaded = true;
                }
            });
            if let Some(cb) = ctx.on_toggle {
                cb.run(key);
            }
        }
    }
}

/// Spawn the async load of `key`'s children into `store` — the single load
/// path shared by [`toggle_by_key`]'s expand branch and the mount effect's
/// eager load of initially-`open()` lazy branches. On completion, splices the
/// loaded children in **only if the node is still `!children_loaded`** at that
/// point: idempotent defense-in-depth against the double-load race (paired
/// with `should_spawn_load` guarding the *spawn* side in `toggle_by_key`), so
/// even if two loads for the same key were ever in flight, children are only
/// ever inserted once.
fn spawn_load_children(
    store: RwSignal<Vec<FlatNode>>,
    loader: TreeLoader,
    key: String,
    on_toggle: Option<Callback<String>>,
) {
    let fut = loader.load(key.clone());
    leptos::task::spawn_local(async move {
        let children = fut.await;
        store.update(|n| {
            if let Some(i) = index_of_key(n, &key)
                && !n[i].children_loaded
            {
                let depth = n[i].depth;
                n[i].children_loaded = true;
                n[i].loading = false;
                insert_children(n, i, flat_children(&children, depth + 1));
            }
        });
        if let Some(cb) = on_toggle {
            cb.run(key);
        }
    });
}

/// Set the selection to `key` and fire `on_selection_change`.
fn select_key(ctx: &Ctx, key: Option<String>) {
    ctx.selected.set(key.clone());
    if let Some(cb) = ctx.on_selection_change {
        cb.run(key);
    }
}

/// Renders one sibling level: the direct children of `parent_key` (or the roots
/// when `None`), as a keyed `<For>`. Keying by [`row_key`] (a content hash of
/// every rendered field, including `loading`) is the epic-wide fix for stale
/// keys — a row re-renders when *any* rendered field changes and keeps its DOM
/// identity otherwise. Expanded branches recurse into a nested `role="group"`.
#[component]
fn TreeLevel(parent_key: Option<String>, ctx: Ctx) -> impl IntoView {
    let store = ctx.store;
    let pk = parent_key.clone();
    // Direct children of this level, recomputed reactively; `<For>` then diffs
    // by content hash so only genuinely-changed rows re-render.
    let siblings = Memo::new(move |_| {
        store.with(|nodes| {
            let idxs = match &pk {
                None => nodes
                    .iter()
                    .enumerate()
                    .filter(|(_, n)| n.depth == 0)
                    .map(|(i, _)| i)
                    .collect::<Vec<_>>(),
                Some(k) => match index_of_key(nodes, k) {
                    Some(i) => child_indices(nodes, i),
                    None => Vec::new(),
                },
            };
            idxs.into_iter()
                .map(|i| nodes[i].clone())
                .collect::<Vec<FlatNode>>()
        })
    });

    view! {
        <For
            each=move || siblings.get()
            key=|node: &FlatNode| row_key(node)
            children=move |node: FlatNode| tree_item(node, ctx.clone())
        />
    }
}

/// Build a single `<li role="treeitem">` (with its row, expander, and — for an
/// expanded branch — a nested `<ul role="group">` recursing via [`TreeLevel`]).
fn tree_item(node: FlatNode, ctx: Ctx) -> AnyView {
    let key = node.key.clone();
    let depth = node.depth;
    let is_branch = node.is_branch;
    let expanded = node.expanded;
    let loading = node.loading;
    let dom_id = key_dom_id(ctx.instance, &key);
    // depth * 1.25rem indent + a small base pad keeps expander glyphs aligned.
    let indent = format!("{}rem", depth as f64 * 1.25 + 0.25);

    let selected = ctx.selected;
    let hover = ctx.hover;

    // Expander: a real <button type="button"> for branches (chevron / spinner),
    // an inert spacer for leaves so labels line up. Deliberately unstyled (no
    // .btn -- a compact 16x16 chevron toggle wants none of .btn's padding or
    // background), so it carries data-pressable="true" for the ldui-audit
    // button-without-btn drift rule (ldui-2e7a).
    let expander = if is_branch {
        let ctx_click = ctx.clone();
        let key_click = key.clone();
        view! {
            <button
                type="button"
                data-pressable="true"
                class="shrink-0 w-4 h-4 flex items-center justify-center text-base-content/50 hover:text-base-content"
                aria-hidden="true"
                tabindex="-1"
                on:click=move |ev| {
                    ev.stop_propagation();
                    toggle_by_key(&ctx_click, key_click.clone());
                }
            >
                {if loading {
                    view! { <span class="loading loading-spinner loading-xs"></span> }.into_any()
                } else {
                    let chevron = if expanded {
                        "w-3 h-3 transition-transform rotate-90"
                    } else {
                        "w-3 h-3 transition-transform"
                    };
                    view! {
                        <svg
                            xmlns="http://www.w3.org/2000/svg"
                            viewBox="0 0 20 20"
                            fill="currentColor"
                            class=chevron
                        >
                            <path
                                fill-rule="evenodd"
                                d="M7.21 14.77a.75.75 0 01.02-1.06L11.168 10 7.23 6.29a.75.75 0 111.04-1.08l4.5 4.25a.75.75 0 010 1.08l-4.5 4.25a.75.75 0 01-1.06-.02z"
                                clip-rule="evenodd"
                            />
                        </svg>
                    }
                        .into_any()
                }}
            </button>
        }
        .into_any()
    } else {
        view! { <span class="inline-block w-4 shrink-0"></span> }.into_any()
    };

    let icon_view = node
        .icon
        .clone()
        .map(|ic| view! { <span class="text-sm shrink-0">{ic}</span> }.into_any())
        .unwrap_or_else(|| ().into_any());

    let label = node.label.clone();

    // Row highlight reacts to selection/hover without re-running the `<For>`.
    let key_class = key.clone();
    let row_class = move || {
        let sel = selected.get().as_deref() == Some(key_class.as_str());
        let hov = hover.get().as_deref() == Some(key_class.as_str());
        merge_classes!(
            "flex items-center gap-2 py-1 pr-2 rounded cursor-pointer select-none",
            if sel {
                "bg-primary/10 text-primary"
            } else if hov {
                "bg-base-200"
            } else {
                ""
            }
        )
    };

    let ctx_row = ctx.clone();
    let key_click = key.clone();
    let on_row_click = move |_| {
        select_key(&ctx_row, Some(key_click.clone()));
        if !is_branch && let Some(cb) = ctx_row.on_activate {
            cb.run(key_click.clone());
        }
    };

    let key_enter = key.clone();
    let on_mouseenter = move |_| hover.set(Some(key_enter.clone()));
    let key_leave = key.clone();
    let on_mouseleave = move |_| {
        if hover.get_untracked().as_deref() == Some(key_leave.as_str()) {
            hover.set(None);
        }
    };

    let key_sel = key.clone();
    let aria_selected = move || (selected.get().as_deref() == Some(key_sel.as_str())).to_string();

    // Children group: only present while this branch is expanded.
    let children_view = if is_branch && expanded {
        let ctx_children = ctx.clone();
        let child_pk = key.clone();
        view! {
            <ul role="group" class="list-none m-0 p-0">
                <TreeLevel parent_key=Some(child_pk) ctx=ctx_children />
            </ul>
        }
        .into_any()
    } else {
        ().into_any()
    };

    view! {
        <li
            id=dom_id
            role="treeitem"
            attr:aria-level=(depth + 1).to_string()
            aria-expanded=is_branch.then(|| expanded.to_string())
            aria-selected=aria_selected
        >
            <div
                class=row_class
                style:padding-left=indent
                on:click=on_row_click
                on:mouseenter=on_mouseenter
                on:mouseleave=on_mouseleave
            >
                {expander}
                {icon_view}
                <span class="truncate text-sm">{label}</span>
            </div>
            {children_view}
        </li>
    }
    .into_any()
}

/// # Tree Component
///
/// A lazy, expandable tree view (file-explorer style) — the general-purpose
/// primitive ported from d2d-ui's `controls::tree::Tree`. daisyUI has no tree
/// element, so this is fully custom Tailwind styling. It supports a recursive
/// node model with **on-demand async child loading**, per-node expand/collapse
/// with expander glyphs and per-depth indentation, hover/selection highlight,
/// an `on_activate` event, and full keyboard navigation over the visible nodes.
///
/// d2d's synchronous `TreeDataSource` maps to the async [`TreeLoader`] closure
/// prop: leave a branch's `children` empty and supply a `loader`, and the
/// branch's children are fetched (with a per-node spinner) on first expand via
/// `spawn_local`. Trees with all data up front simply pass eager `children` and
/// omit the loader. d2d's manual scroll/virtualization and horizontal scrollbar
/// are dropped in favour of native `overflow`; the selected row is kept in view
/// with `Element::scroll_into_view` (as `ResultList` does).
///
/// ## Keyboard (focus the tree, WAI-ARIA tree pattern)
/// - `ArrowDown`/`ArrowUp` — move to the next/previous visible node (clamped).
/// - `ArrowRight` — expand a collapsed branch, else move to its first child.
/// - `ArrowLeft` — collapse an expanded branch, else move to its parent.
/// - `Enter` — activate the selected node (`on_activate`).
/// - `Home`/`End` — jump to the first/last visible node.
///
/// The container carries `role="tree"` + `aria-activedescendant`; each row is a
/// `role="treeitem"` with `aria-level`, `aria-expanded` (branches) and
/// `aria-selected`; expanded branches nest a `role="group"`.
///
/// # Related
/// For an app-level, eager (non-lazy) composite that renders a whole `Vec` of
/// nodes as a daisyUI `menu` without keyboard navigation, see
/// [`crate::widgets::TreeView`]. Prefer **this** `Tree` when you need lazy async
/// loading, keyboard navigation, or ARIA tree semantics; prefer `TreeView` for
/// a quick, fully-materialised, click-only menu tree.
///
/// # Example
/// ```rust,ignore
/// use leptos::prelude::*;
/// use leptos_daisyui_rs::components::{Tree, TreeNode, TreeChild, TreeLoader};
///
/// #[component]
/// fn App() -> impl IntoView {
///     let roots = vec![TreeNode::branch("/", "root").with_icon("📁").open()];
///     let loader = TreeLoader::new(|key: String| async move {
///         vec![
///             TreeChild::branch(format!("{key}sub/"), "sub").with_icon("📁"),
///             TreeChild::leaf(format!("{key}file.rs"), "file.rs").with_icon("📄"),
///         ]
///     });
///     view! {
///         <Tree
///             nodes=Signal::derive(move || roots.clone())
///             loader=loader
///             on_activate=Callback::new(|k: String| leptos::logging::log!("open {k}"))
///         />
///     }
/// }
/// ```
///
/// ## CSS
/// Every literal utility class this component can render (add to `input.css`):
/// ```css
/// @source inline("w-full max-h-96 overflow-auto rounded-box border border-base-300 bg-base-100 p-1");
/// @source inline("outline-none focus:ring-2 focus:ring-primary/50 list-none m-0 p-0");
/// @source inline("flex items-center gap-2 py-1 pr-2 rounded cursor-pointer select-none");
/// @source inline("bg-primary/10 text-primary bg-base-200");
/// @source inline("shrink-0 w-4 h-4 flex items-center justify-center text-base-content/50 hover:text-base-content");
/// @source inline("loading loading-spinner loading-xs w-3 h-3 transition-transform rotate-90");
/// @source inline("inline-block w-4 shrink-0 text-sm truncate");
/// ```
///
/// ## Node References
/// - `node_ref` - References the tree container `<ul>`
///   ([HTMLUListElement](https://developer.mozilla.org/en-US/docs/Web/API/HTMLUListElement))
#[component]
pub fn Tree(
    /// The root-level nodes. Replacing this signal rebuilds the tree and resets
    /// selection/hover (mirrors `ResultList`'s items-replaced behaviour).
    #[prop(optional, into)]
    nodes: Signal<Vec<TreeNode>>,

    /// Async loader supplying a branch's children on first expand. Omit for a
    /// fully eager tree (all `children` provided up front).
    #[prop(optional)]
    loader: Option<TreeLoader>,

    /// Fired when a node is activated (`Enter` key, or click on a leaf) with the
    /// activated node's key.
    #[prop(optional)]
    on_activate: Option<Callback<String>>,

    /// Fired when a branch is expanded or collapsed, with the branch's key.
    #[prop(optional)]
    on_toggle: Option<Callback<String>>,

    /// Fired whenever the highlighted node changes, with its key (or `None`).
    #[prop(optional)]
    on_selection_change: Option<Callback<Option<String>>>,

    /// Additional CSS classes for the tree container.
    #[prop(optional, into)]
    class: &'static str,

    /// Node reference for the tree container `<ul>`.
    #[prop(optional)]
    node_ref: NodeRef<html::Ul>,
) -> impl IntoView {
    let instance = TREE_SEQ.fetch_add(1, Ordering::Relaxed);

    let store = RwSignal::new(Vec::<FlatNode>::new());
    let selected = RwSignal::new(None::<String>);
    let hover = RwSignal::new(None::<String>);

    // Rebuild the flat model whenever the roots signal changes; reset state.
    let effect_loader = loader.clone();
    let effect_on_toggle = on_toggle;
    Effect::new(move |_| {
        let roots = nodes.get();
        store.set(build_flat(&roots));
        selected.set(None);
        hover.set(None);

        // `build_flat` only carries over `TreeNode::open()`'s `expanded` flag —
        // it never fetches, so an initially-open *lazy* branch would otherwise
        // render expanded-but-empty until manually toggled twice. Eagerly spawn
        // the same load path `toggle_by_key` uses for every such branch.
        if let Some(loader) = effect_loader.clone() {
            let to_load: Vec<String> = store.with_untracked(|n| {
                n.iter()
                    .filter(|node| node.expanded && should_spawn_load(node))
                    .map(|node| node.key.clone())
                    .collect()
            });
            if !to_load.is_empty() {
                store.update(|n| {
                    for key in &to_load {
                        if let Some(i) = index_of_key(n, key) {
                            n[i].loading = true;
                        }
                    }
                });
                for key in to_load {
                    spawn_load_children(store, loader.clone(), key, effect_on_toggle);
                }
            }
        }
    });

    let ctx = Ctx {
        instance,
        store,
        selected,
        hover,
        loader,
        on_activate,
        on_toggle,
        on_selection_change,
    };

    // Keep the selected row in view as selection moves (native overflow +
    // scroll_into_view; no manual scroll math).
    Effect::new(move |_| {
        let Some(key) = selected.get() else {
            return;
        };
        if let Some(el) = node_ref.get_untracked() {
            let container = el.unchecked_ref::<web_sys::Element>();
            let selector = format!("#{}", key_dom_id(instance, &key));
            if let Ok(Some(target)) = container.query_selector(&selector) {
                let opts = ScrollIntoViewOptions::new();
                opts.set_block(ScrollLogicalPosition::Nearest);
                target.scroll_into_view_with_scroll_into_view_options(&opts);
            }
        }
    });

    let ctx_key = ctx.clone();
    let on_keydown = move |ev: KeyboardEvent| {
        let nodes_snap = store.get_untracked();
        if nodes_snap.is_empty() {
            return;
        }
        let current = selected
            .get_untracked()
            .as_deref()
            .and_then(|k| index_of_key(&nodes_snap, k));
        let k = ev.key();
        let nav = handle_key(&nodes_snap, current, k.as_str());
        if !matches!(nav, KeyNav::None) {
            ev.prevent_default();
        }
        match nav {
            KeyNav::Move(i) => select_key(&ctx_key, Some(nodes_snap[i].key.clone())),
            KeyNav::Expand(i) | KeyNav::Collapse(i) => {
                toggle_by_key(&ctx_key, nodes_snap[i].key.clone())
            }
            KeyNav::Activate(i) => {
                let key = nodes_snap[i].key.clone();
                select_key(&ctx_key, Some(key.clone()));
                if let Some(cb) = ctx_key.on_activate {
                    cb.run(key);
                }
            }
            KeyNav::None => {}
        }
    };

    let aria_active = move || selected.get().map(|k| key_dom_id(instance, &k));

    let ctx_root = ctx.clone();
    view! {
        <ul
            node_ref=node_ref
            role="tree"
            tabindex="0"
            aria-activedescendant=aria_active
            class=move || {
                merge_classes!(
                    "w-full max-h-96 overflow-auto rounded-box border border-base-300 bg-base-100 p-1 list-none m-0 outline-none focus:ring-2 focus:ring-primary/50",
                    class
                )
            }
            on:keydown=on_keydown
        >
            <TreeLevel parent_key=None ctx=ctx_root />
        </ul>
    }
}
