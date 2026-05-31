use leptos::prelude::*;

/// A node in the tree hierarchy.
#[derive(Clone, Debug, PartialEq)]
pub struct TreeNode {
    /// Unique identifier for this node.
    pub id: String,
    /// Display label for this node.
    pub label: String,
    /// Child nodes nested under this one.
    pub children: Vec<TreeNode>,
    /// Whether this node starts expanded.
    pub expanded: bool,
    /// Optional icon CSS class (e.g., a heroicon or emoji).
    pub icon: Option<String>,
}

/// A flattened representation of a tree node used for rendering.
#[derive(Clone)]
struct FlatNode {
    id: String,
    label: String,
    icon: Option<String>,
    depth: usize,
    has_children: bool,
    expanded: RwSignal<bool>,
    /// Indices of direct children in the flat list.
    child_indices: Vec<usize>,
}

/// Flatten a tree structure into a linear list for rendering,
/// returning signals for expansion state.
fn flatten_tree(nodes: &[TreeNode], depth: usize, flat: &mut Vec<FlatNode>) {
    for node in nodes {
        let idx = flat.len();
        let expanded = RwSignal::new(node.expanded);
        flat.push(FlatNode {
            id: node.id.clone(),
            label: node.label.clone(),
            icon: node.icon.clone(),
            depth,
            has_children: !node.children.is_empty(),
            expanded,
            child_indices: Vec::new(),
        });
        let children_start = flat.len();
        flatten_tree(&node.children, depth + 1, flat);
        let children_end = flat.len();
        // Record direct child indices (only immediate children at depth+1)
        let child_indices: Vec<usize> = (children_start..children_end)
            .filter(|&i| flat[i].depth == depth + 1)
            .collect();
        flat[idx].child_indices = child_indices;
    }
}

/// Check if a node at the given index should be visible based on ancestor
/// expansion state.
fn is_visible(flat: &[FlatNode], index: usize) -> bool {
    if flat[index].depth == 0 {
        return true;
    }
    // Walk backwards to find the parent and check if it's expanded
    let target_depth = flat[index].depth;
    for i in (0..index).rev() {
        if flat[i].depth == target_depth - 1 {
            // This is the parent
            if !flat[i].expanded.get_untracked() {
                return false;
            }
            return is_visible(flat, i);
        }
    }
    true
}

/// Recursive tree view component for hierarchical data such as org charts
/// or asset dependency trees.
///
/// Each node can be expanded or collapsed by clicking. Indentation increases
/// with depth, and optional icons are displayed before the label.
#[component]
pub fn TreeView(
    /// The root-level nodes to display.
    nodes: Vec<TreeNode>,
) -> impl IntoView {
    // Flatten tree for non-recursive rendering
    let mut flat_nodes = Vec::new();
    flatten_tree(&nodes, 0, &mut flat_nodes);
    let flat_nodes = StoredValue::new(flat_nodes);

    // Collect all expansion signals for reactivity trigger
    let expansion_signals: Vec<RwSignal<bool>> =
        flat_nodes.get_value().iter().map(|n| n.expanded).collect();
    let expansion_signals = StoredValue::new(expansion_signals);

    view! {
        <div class="tree-view">
            <ul class="menu menu-xs w-full">
                {move || {
                    // Read all expansion signals to trigger reactivity
                    let signals = expansion_signals.get_value();
                    for sig in &signals {
                        let _ = sig.get();
                    }

                    let all_nodes = flat_nodes.get_value();
                    all_nodes.iter().enumerate().filter_map(|(idx, node)| {
                        // Check visibility by walking up the ancestor chain
                        if !is_node_visible(&all_nodes, idx) {
                            return None;
                        }

                        let padding_left = format!("{}rem", node.depth as f64 * 1.25);
                        let label = node.label.clone();
                        let icon = node.icon.clone();
                        let has_children = node.has_children;
                        let expanded_signal = node.expanded;

                        let chevron = if has_children {
                            let is_expanded = expanded_signal.get_untracked();
                            let chevron_class = if is_expanded {
                                "inline-block w-4 text-base-content/50 rotate-90"
                            } else {
                                "inline-block w-4 text-base-content/50"
                            };
                            view! {
                                <span class=chevron_class>
                                    <svg xmlns="http://www.w3.org/2000/svg" viewBox="0 0 20 20" fill="currentColor" class="w-4 h-4">
                                        <path fill-rule="evenodd" d="M7.21 14.77a.75.75 0 01.02-1.06L11.168 10 7.23 6.29a.75.75 0 111.04-1.08l4.5 4.25a.75.75 0 010 1.08l-4.5 4.25a.75.75 0 01-1.06-.02z" clip-rule="evenodd" />
                                    </svg>
                                </span>
                            }.into_any()
                        } else {
                            view! { <span class="inline-block w-4"></span> }.into_any()
                        };

                        let icon_view = icon.map(|ic| view! {
                            <span class="text-sm">{ic}</span>
                        }).into_any();

                        Some(view! {
                            <li>
                                <div
                                    class="flex items-center gap-2 cursor-pointer select-none rounded px-2 py-1 hover:bg-base-200 transition-colors"
                                    style:padding-left=padding_left
                                    on:click=move |_| {
                                        if has_children {
                                            expanded_signal.update(|v| *v = !*v);
                                        }
                                    }
                                >
                                    {chevron}
                                    {icon_view}
                                    <span class="text-sm font-medium">{label}</span>
                                </div>
                            </li>
                        })
                    }).collect::<Vec<_>>()
                }}
            </ul>
        </div>
    }
}

/// Check if a node at the given index is visible (all ancestors expanded).
fn is_node_visible(flat: &[FlatNode], index: usize) -> bool {
    let node = &flat[index];
    if node.depth == 0 {
        return true;
    }
    // Walk backwards to find the parent (first node with depth - 1)
    let parent_depth = node.depth - 1;
    for i in (0..index).rev() {
        if flat[i].depth == parent_depth {
            // Found the parent
            if !flat[i].expanded.get_untracked() {
                return false;
            }
            // Check the parent's visibility recursively
            return is_node_visible(flat, i);
        }
    }
    true
}
