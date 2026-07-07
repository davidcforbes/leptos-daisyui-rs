//! Data model + pure logic for the [`Tree`](super::Tree) component.
//!
//! Ported from d2d-ui's `controls::tree::Tree` — a self-painting Direct2D
//! control that owns a lazily-expanded node model and paints its own rows,
//! expander glyphs, hover/selection highlight and a bespoke horizontal
//! scrollbar. None of the renderer-specific machinery ports: Direct2D drawing,
//! DPI scaling, pixel hit-testing, manual row-height measurement and the
//! hand-rolled scroll engine are all replaced by the browser's native layout,
//! `overflow`, and `Element::scroll_into_view`.
//!
//! What *does* port is the model + the pure logic that is the heart of the
//! component: flattening the tree to the currently-visible rows, and the
//! keyboard-navigation index math over that visible order. Those live here as
//! pure functions so they are unit-testable without a DOM (see `tests.rs`).
//!
//! d2d's synchronous [`TreeDataSource::children`] trait becomes an **async**
//! [`TreeLoader`] closure prop: a host supplies the children for an opaque node
//! key on demand, and the [`Tree`](super::Tree) component awaits it via
//! `spawn_local` on expand, showing a per-node loading state until it resolves.
//! This is the Leptos-idiomatic shape — load-on-expand with `spawn_local`
//! rather than a single up-front `Resource`, because each branch is fetched
//! independently and only when the user reveals it.

use std::hash::{DefaultHasher, Hash, Hasher};

/// A recursive input node handed to [`Tree`](super::Tree) as its initial roots.
///
/// For **eager** trees, populate [`children`](Self::children) up front. For
/// **lazy** trees, leave `children` empty on a branch (`is_branch == true`) and
/// supply a [`TreeLoader`]; the branch's children are then fetched on first
/// expand.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct TreeNode {
    /// Opaque, host-assigned identifier — must be unique across the tree. Used
    /// as the stable selection key and passed to the [`TreeLoader`] to fetch
    /// this node's children.
    pub key: String,
    /// Display text.
    pub label: String,
    /// Whether this node can be expanded (has, or can load, children).
    pub is_branch: bool,
    /// Optional leading icon (emoji or a text glyph rendered before the label).
    pub icon: Option<String>,
    /// Whether this node starts expanded.
    pub expanded: bool,
    /// Pre-loaded children (eager). Leave empty for a lazily-loaded branch.
    pub children: Vec<TreeNode>,
}

impl TreeNode {
    /// A leaf node (cannot be expanded).
    pub fn leaf(key: impl Into<String>, label: impl Into<String>) -> Self {
        Self {
            key: key.into(),
            label: label.into(),
            is_branch: false,
            icon: None,
            expanded: false,
            children: Vec::new(),
        }
    }

    /// A branch node (can be expanded). With no [`children`](Self::children) it
    /// is a lazy branch resolved by the [`TreeLoader`] on first expand.
    pub fn branch(key: impl Into<String>, label: impl Into<String>) -> Self {
        Self {
            key: key.into(),
            label: label.into(),
            is_branch: true,
            icon: None,
            expanded: false,
            children: Vec::new(),
        }
    }

    /// Attach a leading icon.
    pub fn with_icon(mut self, icon: impl Into<String>) -> Self {
        self.icon = Some(icon.into());
        self
    }

    /// Attach eager children (also forces `is_branch = true`).
    pub fn with_children(mut self, children: Vec<TreeNode>) -> Self {
        self.is_branch = true;
        self.children = children;
        self
    }

    /// Start this node expanded.
    pub fn open(mut self) -> Self {
        self.expanded = true;
        self
    }
}

/// A child entry returned by a [`TreeLoader`] — the async analogue of d2d's
/// `TreeChild`.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct TreeChild {
    /// Opaque, host-assigned identifier (unique across the tree).
    pub key: String,
    /// Display text.
    pub label: String,
    /// Whether this child is itself a branch.
    pub is_branch: bool,
    /// Optional leading icon.
    pub icon: Option<String>,
}

impl TreeChild {
    /// A leaf child.
    pub fn leaf(key: impl Into<String>, label: impl Into<String>) -> Self {
        Self {
            key: key.into(),
            label: label.into(),
            is_branch: false,
            icon: None,
        }
    }

    /// A branch child.
    pub fn branch(key: impl Into<String>, label: impl Into<String>) -> Self {
        Self {
            key: key.into(),
            label: label.into(),
            is_branch: true,
            icon: None,
        }
    }

    /// Attach a leading icon.
    pub fn with_icon(mut self, icon: impl Into<String>) -> Self {
        self.icon = Some(icon.into());
        self
    }
}

/// The flattened, depth-first render/navigation model — the single source of
/// truth held in the component's `RwSignal`. Every field is render-relevant, so
/// [`row_key`] hashes the whole struct for the `<For>`-equivalent key (the
/// epic-wide "stale key" lesson: keys must change when *any* rendered field
/// changes — e.g. `loading` flipping while children fetch).
#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub struct FlatNode {
    /// Stable, unique node identifier (see [`TreeNode::key`]).
    pub key: String,
    /// Display text.
    pub label: String,
    /// Whether this node can be expanded.
    pub is_branch: bool,
    /// Optional leading icon.
    pub icon: Option<String>,
    /// Depth from the roots (roots are depth 0).
    pub depth: usize,
    /// Whether this branch is currently expanded.
    pub expanded: bool,
    /// `true` once this branch's children have been loaded (or it had eager
    /// children). Prevents re-fetching on every expand.
    pub children_loaded: bool,
    /// `true` while this branch's children are being fetched asynchronously.
    pub loading: bool,
}

/// Flatten recursive input roots into the depth-first [`FlatNode`] list.
pub fn build_flat(roots: &[TreeNode]) -> Vec<FlatNode> {
    let mut out = Vec::new();
    for node in roots {
        push_flat(node, 0, &mut out);
    }
    out
}

fn push_flat(node: &TreeNode, depth: usize, out: &mut Vec<FlatNode>) {
    // A branch with eager children is considered loaded; a lazy branch (no
    // children provided) is not, so it fetches on first expand.
    let children_loaded = !node.is_branch || !node.children.is_empty();
    out.push(FlatNode {
        key: node.key.clone(),
        label: node.label.clone(),
        is_branch: node.is_branch,
        icon: node.icon.clone(),
        depth,
        expanded: node.expanded,
        children_loaded,
        loading: false,
    });
    for child in &node.children {
        push_flat(child, depth + 1, out);
    }
}

/// Convert loaded [`TreeChild`]ren into [`FlatNode`]s at `depth` (used when a
/// lazy branch resolves; children are always freshly loaded, never expanded).
pub fn flat_children(children: &[TreeChild], depth: usize) -> Vec<FlatNode> {
    children
        .iter()
        .map(|c| FlatNode {
            key: c.key.clone(),
            label: c.label.clone(),
            is_branch: c.is_branch,
            icon: c.icon.clone(),
            depth,
            expanded: false,
            // A freshly-loaded branch child has not itself been loaded yet.
            children_loaded: !c.is_branch,
            loading: false,
        })
        .collect()
}

/// Index of the node with `key`, or `None`. Selection is tracked by key (not
/// index) so it survives the index shifts caused by lazy insertion/collapse.
pub fn index_of_key(nodes: &[FlatNode], key: &str) -> Option<usize> {
    nodes.iter().position(|n| n.key == key)
}

/// The indices of the currently **visible** nodes, in top-to-bottom order: a
/// node is visible when every ancestor is expanded. Descendants of a collapsed
/// branch are skipped. This is the flatten-visible-nodes function the keyboard
/// navigation operates over.
pub fn visible_indices(nodes: &[FlatNode]) -> Vec<usize> {
    let mut out = Vec::new();
    let mut i = 0;
    while i < nodes.len() {
        out.push(i);
        if nodes[i].is_branch && nodes[i].expanded {
            // Descend: the next node is this branch's first child.
            i += 1;
        } else {
            // Skip the whole subtree of a collapsed/leaf node.
            let depth = nodes[i].depth;
            i += 1;
            while i < nodes.len() && nodes[i].depth > depth {
                i += 1;
            }
        }
    }
    out
}

/// Direct child indices of `idx` (nodes at `depth + 1` within its contiguous
/// subtree), in order.
pub fn child_indices(nodes: &[FlatNode], idx: usize) -> Vec<usize> {
    let depth = nodes[idx].depth;
    let mut out = Vec::new();
    let mut i = idx + 1;
    while i < nodes.len() && nodes[i].depth > depth {
        if nodes[i].depth == depth + 1 {
            out.push(i);
        }
        i += 1;
    }
    out
}

/// Index of `idx`'s first direct child, or `None` if it has no children loaded.
pub fn first_child_index(nodes: &[FlatNode], idx: usize) -> Option<usize> {
    let next = idx + 1;
    if next < nodes.len() && nodes[next].depth == nodes[idx].depth + 1 {
        Some(next)
    } else {
        None
    }
}

/// Index of `idx`'s parent (the nearest earlier node one level shallower), or
/// `None` for a root.
pub fn parent_index(nodes: &[FlatNode], idx: usize) -> Option<usize> {
    let depth = nodes[idx].depth;
    if depth == 0 {
        return None;
    }
    (0..idx).rev().find(|&i| nodes[i].depth == depth - 1)
}

/// The insertion point for `idx`'s newly-loaded children: just after `idx`'s
/// entire existing subtree (so re-loads or eager grandchildren stay ordered).
fn insert_point(nodes: &[FlatNode], idx: usize) -> usize {
    let depth = nodes[idx].depth;
    (idx + 1..nodes.len())
        .find(|&i| nodes[i].depth <= depth)
        .unwrap_or(nodes.len())
}

/// Splice freshly-loaded `children` into `nodes` as the children of `parent_idx`.
pub fn insert_children(nodes: &mut Vec<FlatNode>, parent_idx: usize, children: Vec<FlatNode>) {
    let at = insert_point(nodes, parent_idx);
    nodes.splice(at..at, children);
}

/// The result of interpreting a key press over the visible tree — a pure
/// intent the component then applies (moving selection, or expanding/collapsing
/// which may trigger an async load).
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum KeyNav {
    /// Move the selection to this node index.
    Move(usize),
    /// Expand the branch at this node index (may lazily load children).
    Expand(usize),
    /// Collapse the branch at this node index.
    Collapse(usize),
    /// Activate the node at this index (`Enter`).
    Activate(usize),
    /// No change.
    None,
}

/// Interpret a key press given the full node list and the currently-selected
/// node index. Pure — the heart of the component's keyboard navigation.
///
/// - `ArrowDown`/`ArrowUp`: move to the next/previous **visible** node
///   (clamped at the ends, no wraparound). From no selection, both select the
///   first visible node.
/// - `ArrowRight`: on a collapsed branch, expand it; on an expanded branch,
///   move to its first child; on a leaf, nothing.
/// - `ArrowLeft`: on an expanded branch, collapse it; otherwise move to the
///   parent.
/// - `Enter`: activate the selected node.
/// - `Home`/`End`: jump to the first/last visible node.
pub fn handle_key(nodes: &[FlatNode], current: Option<usize>, key: &str) -> KeyNav {
    let visible = visible_indices(nodes);
    if visible.is_empty() {
        return KeyNav::None;
    }
    let pos = current.and_then(|c| visible.iter().position(|&v| v == c));

    match key {
        "ArrowDown" => match pos {
            Some(p) if p + 1 < visible.len() => KeyNav::Move(visible[p + 1]),
            Some(p) => KeyNav::Move(visible[p]),
            None => KeyNav::Move(visible[0]),
        },
        "ArrowUp" => match pos {
            Some(p) if p > 0 => KeyNav::Move(visible[p - 1]),
            Some(p) => KeyNav::Move(visible[p]),
            None => KeyNav::Move(visible[0]),
        },
        "ArrowRight" => match current {
            Some(c) => {
                let node = &nodes[c];
                if node.is_branch && !node.expanded {
                    KeyNav::Expand(c)
                } else if node.is_branch && node.expanded {
                    match first_child_index(nodes, c) {
                        Some(child) => KeyNav::Move(child),
                        None => KeyNav::None,
                    }
                } else {
                    KeyNav::None
                }
            }
            None => KeyNav::Move(visible[0]),
        },
        "ArrowLeft" => match current {
            Some(c) => {
                let node = &nodes[c];
                if node.is_branch && node.expanded {
                    KeyNav::Collapse(c)
                } else {
                    match parent_index(nodes, c) {
                        Some(parent) => KeyNav::Move(parent),
                        None => KeyNav::None,
                    }
                }
            }
            None => KeyNav::None,
        },
        "Enter" => match current {
            Some(c) => KeyNav::Activate(c),
            None => KeyNav::None,
        },
        "Home" => KeyNav::Move(visible[0]),
        "End" => KeyNav::Move(visible[visible.len() - 1]),
        _ => KeyNav::None,
    }
}

/// Content fingerprint of a flattened node, used as its render key so a row
/// re-renders whenever *any* of its rendered fields change (expansion,
/// selection-independent state, and crucially the `loading` flag that flips
/// while async children fetch).
pub fn row_key(node: &FlatNode) -> u64 {
    let mut hasher = DefaultHasher::new();
    node.hash(&mut hasher);
    hasher.finish()
}

/// Async loader that supplies a branch's children on demand — the Leptos
/// analogue of d2d's `TreeDataSource`. Construct with [`TreeLoader::new`] from
/// any `Fn(String) -> impl Future<Output = Vec<TreeChild>>`.
///
/// ```rust,ignore
/// let loader = TreeLoader::new(|key: String| async move {
///     // fetch children for `key` (e.g. an HTTP request) ...
///     vec![TreeChild::leaf(format!("{key}/a"), "a")]
/// });
/// ```
#[derive(Clone)]
pub struct TreeLoader {
    inner: std::sync::Arc<dyn Fn(String) -> TreeChildrenFuture + Send + Sync>,
}

/// The boxed future a [`TreeLoader`] returns. It is *not* required to be `Send`
/// (CSR: awaited via `spawn_local` on the main thread) — only the loader
/// closure itself is `Send + Sync` so a `Tree` can live in Leptos's threadsafe
/// reactive graph.
pub type TreeChildrenFuture = std::pin::Pin<Box<dyn std::future::Future<Output = Vec<TreeChild>>>>;

impl TreeLoader {
    /// Wrap a child-fetching closure. The closure must be `Send + Sync` (its
    /// returned future need not be); most CSR loaders — e.g.
    /// `|key| async move { ... }` capturing only `Copy`/owned data — satisfy
    /// this automatically.
    pub fn new<F, Fut>(f: F) -> Self
    where
        F: Fn(String) -> Fut + Send + Sync + 'static,
        Fut: std::future::Future<Output = Vec<TreeChild>> + 'static,
    {
        Self {
            inner: std::sync::Arc::new(move |key| Box::pin(f(key)) as TreeChildrenFuture),
        }
    }

    /// Begin loading the children of `key`.
    pub fn load(&self, key: String) -> TreeChildrenFuture {
        (self.inner)(key)
    }
}

impl std::fmt::Debug for TreeLoader {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str("TreeLoader(..)")
    }
}
