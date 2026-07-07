use crate::core::{ContentLayout, Section};
use leptos::prelude::*;
use leptos_daisyui_rs::components::*;

#[component]
pub fn TreeDemo() -> impl IntoView {
    let (activated, set_activated) = signal(String::from("None"));
    let (highlighted, set_highlighted) = signal(String::from("None"));

    // ── Lazy tree: children are fetched on first expand. ──
    // A synthetic loader that fabricates a few children per branch key; a real
    // host would issue an HTTP / filesystem request here and `.await` it.
    let lazy_roots = vec![
        TreeNode::branch("/", "workspace").with_icon("📁").open(),
    ];
    let loader = TreeLoader::new(|key: String| async move {
        // Depth-limit the synthetic tree so it terminates.
        let depth = key.matches('/').count();
        if depth >= 4 {
            return vec![TreeChild::leaf(format!("{key}readme.md"), "readme.md").with_icon("📄")];
        }
        vec![
            TreeChild::branch(format!("{key}src/"), "src").with_icon("📁"),
            TreeChild::branch(format!("{key}assets/"), "assets").with_icon("📁"),
            TreeChild::leaf(format!("{key}Cargo.toml"), "Cargo.toml").with_icon("📄"),
            TreeChild::leaf(format!("{key}README.md"), "README.md").with_icon("📄"),
        ]
    });

    // ── Eager tree: all nodes supplied up front, no loader. ──
    let eager_roots = vec![
        TreeNode::branch("animals", "Animals")
            .with_icon("🐾")
            .open()
            .with_children(vec![
                TreeNode::branch("mammals", "Mammals").open().with_children(vec![
                    TreeNode::leaf("dog", "Dog").with_icon("🐕"),
                    TreeNode::leaf("cat", "Cat").with_icon("🐈"),
                ]),
                TreeNode::branch("birds", "Birds").with_children(vec![
                    TreeNode::leaf("owl", "Owl").with_icon("🦉"),
                    TreeNode::leaf("duck", "Duck").with_icon("🦆"),
                ]),
            ]),
        TreeNode::leaf("mineral", "Mineral").with_icon("💎"),
    ];

    view! {
        <ContentLayout
            title="Tree"
            description="Lazy, expandable, keyboard-navigable tree view (file-explorer style) with async child loading"
        >
            <Section title="Lazy Loading (async children on expand)">
                <p class="text-sm opacity-70 mb-2">
                    "Click a folder's chevron to expand — children load asynchronously (a spinner shows while fetching). "
                    "Focus the tree and use "
                    <kbd class="kbd kbd-sm">"↑"</kbd> " " <kbd class="kbd kbd-sm">"↓"</kbd>
                    " to move, " <kbd class="kbd kbd-sm">"→"</kbd> " / " <kbd class="kbd kbd-sm">"←"</kbd>
                    " to expand/collapse, and " <kbd class="kbd kbd-sm">"Enter"</kbd> " to activate."
                </p>
                <div class="alert alert-info mb-4">
                    <span>
                        "Highlighted: " <strong>{move || highlighted.get()}</strong>
                        " | Activated: " <strong>{move || activated.get()}</strong>
                    </span>
                </div>
                <div class="max-w-md">
                    <Tree
                        nodes=Signal::derive(move || lazy_roots.clone())
                        loader=loader
                        on_selection_change=Callback::new(move |k: Option<String>| {
                            set_highlighted.set(k.unwrap_or_else(|| "None".to_string()));
                        })
                        on_activate=Callback::new(move |k: String| {
                            set_activated.set(k);
                        })
                    />
                </div>
            </Section>

            <Section title="Eager Tree (all data up front)">
                <div class="max-w-md">
                    <Tree
                        nodes=Signal::derive(move || eager_roots.clone())
                        on_activate=Callback::new(move |k: String| {
                            set_activated.set(k);
                        })
                    />
                </div>
            </Section>

            <Section title="Features">
                <ul class="list-disc list-inside space-y-1 text-base-content/70">
                    <li>"Lazy async child loading via a "<code>"TreeLoader"</code>" closure (per-node spinner while fetching)"</li>
                    <li>"Eager mode: pass "<code>"TreeNode::with_children"</code>" and omit the loader"</li>
                    <li>"Arrow keys navigate visible nodes; Right/Left expand-collapse or move to child/parent"</li>
                    <li>"Enter activates; Home / End jump to the first / last visible node"</li>
                    <li>"Selected node auto-scrolls into view"</li>
                    <li>"WAI-ARIA tree pattern: role=tree/treeitem/group, aria-level, aria-expanded, aria-selected, aria-activedescendant"</li>
                </ul>
            </Section>
        </ContentLayout>
    }
}
