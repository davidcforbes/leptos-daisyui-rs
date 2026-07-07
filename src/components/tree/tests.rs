use super::*;

/// Build the canonical fixture used across the nav tests:
///
/// ```text
/// root        (branch, expanded)   idx 0  depth 0
///   dir_a     (branch, expanded)   idx 1  depth 1
///     leaf_x  (leaf)               idx 2  depth 2
///   file_b    (leaf)               idx 3  depth 1
/// orphan      (leaf)               idx 4  depth 0
/// ```
fn fixture() -> Vec<FlatNode> {
    build_flat(&[
        TreeNode::branch("root", "root")
            .open()
            .with_children(vec![
                TreeNode::branch("dir_a", "dir_a")
                    .open()
                    .with_children(vec![TreeNode::leaf("leaf_x", "leaf_x")]),
                TreeNode::leaf("file_b", "file_b"),
            ]),
        TreeNode::leaf("orphan", "orphan"),
    ])
}

// ── build_flat ──────────────────────────────────────────────────────────

#[test]
fn build_flat_assigns_depth_first_order_and_depths() {
    let nodes = fixture();
    let shape: Vec<(&str, usize)> = nodes.iter().map(|n| (n.key.as_str(), n.depth)).collect();
    assert_eq!(
        shape,
        vec![
            ("root", 0),
            ("dir_a", 1),
            ("leaf_x", 2),
            ("file_b", 1),
            ("orphan", 0),
        ]
    );
}

#[test]
fn build_flat_marks_eager_branch_loaded_but_lazy_branch_not() {
    let nodes = build_flat(&[
        TreeNode::branch("lazy", "lazy"), // no children provided
        TreeNode::branch("eager", "eager").with_children(vec![TreeNode::leaf("c", "c")]),
        TreeNode::leaf("plain", "plain"),
    ]);
    assert!(!nodes[0].children_loaded, "lazy branch not yet loaded");
    assert!(nodes[1].children_loaded, "eager branch already loaded");
    assert!(nodes[3].children_loaded, "leaf is trivially 'loaded'");
}

// ── visible_indices ─────────────────────────────────────────────────────

#[test]
fn visible_indices_all_visible_when_expanded() {
    let nodes = fixture();
    assert_eq!(visible_indices(&nodes), vec![0, 1, 2, 3, 4]);
}

#[test]
fn visible_indices_hides_descendants_of_collapsed_branch() {
    let mut nodes = fixture();
    nodes[1].expanded = false; // collapse dir_a → leaf_x hidden
    assert_eq!(visible_indices(&nodes), vec![0, 1, 3, 4]);
}

#[test]
fn visible_indices_collapsed_root_hides_whole_subtree() {
    let mut nodes = fixture();
    nodes[0].expanded = false; // collapse root
    assert_eq!(visible_indices(&nodes), vec![0, 4]);
}

// ── first_child / parent / child_indices ────────────────────────────────

#[test]
fn first_child_index_of_branch_and_leaf() {
    let nodes = fixture();
    assert_eq!(first_child_index(&nodes, 0), Some(1)); // root → dir_a
    assert_eq!(first_child_index(&nodes, 1), Some(2)); // dir_a → leaf_x
    assert_eq!(first_child_index(&nodes, 2), None); // leaf_x has none
    assert_eq!(first_child_index(&nodes, 3), None); // file_b leaf
}

#[test]
fn parent_index_walks_up_one_level() {
    let nodes = fixture();
    assert_eq!(parent_index(&nodes, 0), None); // root
    assert_eq!(parent_index(&nodes, 1), Some(0)); // dir_a → root
    assert_eq!(parent_index(&nodes, 2), Some(1)); // leaf_x → dir_a
    assert_eq!(parent_index(&nodes, 3), Some(0)); // file_b → root
    assert_eq!(parent_index(&nodes, 4), None); // orphan is a root
}

#[test]
fn child_indices_returns_direct_children_only() {
    let nodes = fixture();
    assert_eq!(child_indices(&nodes, 0), vec![1, 3]); // root: dir_a, file_b (not leaf_x)
    assert_eq!(child_indices(&nodes, 1), vec![2]); // dir_a: leaf_x
    assert_eq!(child_indices(&nodes, 2), Vec::<usize>::new());
}

#[test]
fn index_of_key_finds_and_misses() {
    let nodes = fixture();
    assert_eq!(index_of_key(&nodes, "file_b"), Some(3));
    assert_eq!(index_of_key(&nodes, "nope"), None);
}

// ── insert_children (lazy load splice) ──────────────────────────────────

#[test]
fn insert_children_splices_after_parent() {
    let mut nodes = build_flat(&[
        TreeNode::branch("lazy", "lazy"),
        TreeNode::leaf("sibling", "sibling"),
    ]);
    let kids = flat_children(
        &[TreeChild::leaf("k1", "k1"), TreeChild::branch("k2", "k2")],
        1,
    );
    insert_children(&mut nodes, 0, kids);
    let keys: Vec<&str> = nodes.iter().map(|n| n.key.as_str()).collect();
    assert_eq!(keys, vec!["lazy", "k1", "k2", "sibling"]);
    assert_eq!(nodes[1].depth, 1);
    assert_eq!(nodes[2].depth, 1);
    assert!(!nodes[2].children_loaded, "loaded branch child is itself lazy");
}

// ── handle_key: ArrowDown / ArrowUp ─────────────────────────────────────

#[test]
fn arrow_down_moves_to_next_visible_and_clamps() {
    let nodes = fixture(); // visible: 0,1,2,3,4
    assert_eq!(handle_key(&nodes, Some(0), "ArrowDown"), KeyNav::Move(1));
    assert_eq!(handle_key(&nodes, Some(2), "ArrowDown"), KeyNav::Move(3));
    assert_eq!(handle_key(&nodes, Some(4), "ArrowDown"), KeyNav::Move(4)); // clamp
    assert_eq!(handle_key(&nodes, None, "ArrowDown"), KeyNav::Move(0)); // first
}

#[test]
fn arrow_down_skips_collapsed_subtree() {
    let mut nodes = fixture();
    nodes[1].expanded = false; // visible: 0,1,3,4
    // from dir_a (idx 1) down should land on file_b (idx 3), skipping hidden leaf_x
    assert_eq!(handle_key(&nodes, Some(1), "ArrowDown"), KeyNav::Move(3));
}

#[test]
fn arrow_up_moves_to_prev_visible_and_clamps() {
    let nodes = fixture();
    assert_eq!(handle_key(&nodes, Some(3), "ArrowUp"), KeyNav::Move(2));
    assert_eq!(handle_key(&nodes, Some(0), "ArrowUp"), KeyNav::Move(0)); // clamp
    assert_eq!(handle_key(&nodes, None, "ArrowUp"), KeyNav::Move(0));
}

// ── handle_key: ArrowRight ──────────────────────────────────────────────

#[test]
fn arrow_right_expands_collapsed_branch() {
    let mut nodes = fixture();
    nodes[1].expanded = false;
    assert_eq!(handle_key(&nodes, Some(1), "ArrowRight"), KeyNav::Expand(1));
}

#[test]
fn arrow_right_on_expanded_branch_moves_to_first_child() {
    let nodes = fixture();
    assert_eq!(handle_key(&nodes, Some(1), "ArrowRight"), KeyNav::Move(2));
}

#[test]
fn arrow_right_on_leaf_does_nothing() {
    let nodes = fixture();
    assert_eq!(handle_key(&nodes, Some(3), "ArrowRight"), KeyNav::None);
}

// ── handle_key: ArrowLeft ───────────────────────────────────────────────

#[test]
fn arrow_left_collapses_expanded_branch() {
    let nodes = fixture();
    assert_eq!(handle_key(&nodes, Some(1), "ArrowLeft"), KeyNav::Collapse(1));
}

#[test]
fn arrow_left_on_leaf_moves_to_parent() {
    let nodes = fixture();
    assert_eq!(handle_key(&nodes, Some(2), "ArrowLeft"), KeyNav::Move(1)); // leaf_x → dir_a
    assert_eq!(handle_key(&nodes, Some(3), "ArrowLeft"), KeyNav::Move(0)); // file_b → root
}

#[test]
fn arrow_left_on_collapsed_branch_moves_to_parent() {
    let mut nodes = fixture();
    nodes[1].expanded = false; // dir_a collapsed
    assert_eq!(handle_key(&nodes, Some(1), "ArrowLeft"), KeyNav::Move(0)); // dir_a → root
}

#[test]
fn arrow_left_on_root_does_nothing() {
    let nodes = fixture();
    let mut nodes = nodes;
    nodes[0].expanded = false; // collapsed root, no parent
    assert_eq!(handle_key(&nodes, Some(0), "ArrowLeft"), KeyNav::None);
}

// ── handle_key: Enter / Home / End / misc ───────────────────────────────

#[test]
fn enter_activates_current() {
    let nodes = fixture();
    assert_eq!(handle_key(&nodes, Some(3), "Enter"), KeyNav::Activate(3));
    assert_eq!(handle_key(&nodes, None, "Enter"), KeyNav::None);
}

#[test]
fn home_and_end_jump_to_first_and_last_visible() {
    let mut nodes = fixture();
    nodes[0].expanded = false; // visible: 0,4
    assert_eq!(handle_key(&nodes, Some(4), "Home"), KeyNav::Move(0));
    assert_eq!(handle_key(&nodes, Some(0), "End"), KeyNav::Move(4));
}

#[test]
fn unhandled_key_is_none() {
    let nodes = fixture();
    assert_eq!(handle_key(&nodes, Some(0), "a"), KeyNav::None);
}

#[test]
fn handle_key_on_empty_tree_is_none() {
    let nodes: Vec<FlatNode> = Vec::new();
    assert_eq!(handle_key(&nodes, None, "ArrowDown"), KeyNav::None);
}

// ── end-to-end nav sequence (mirrors d2d's visible-row semantics) ───────

#[test]
fn keyboard_nav_sequence_expand_collapse_move() {
    // Start with dir_a collapsed: visible 0,1,3,4.
    let mut nodes = fixture();
    nodes[1].expanded = false;

    // On dir_a, ArrowRight expands it.
    assert_eq!(handle_key(&nodes, Some(1), "ArrowRight"), KeyNav::Expand(1));
    nodes[1].expanded = true; // apply → visible 0,1,2,3,4

    // ArrowRight again moves into first child (leaf_x).
    assert_eq!(handle_key(&nodes, Some(1), "ArrowRight"), KeyNav::Move(2));

    // From leaf_x, ArrowLeft returns to parent dir_a.
    assert_eq!(handle_key(&nodes, Some(2), "ArrowLeft"), KeyNav::Move(1));

    // On dir_a, ArrowLeft collapses it.
    assert_eq!(handle_key(&nodes, Some(1), "ArrowLeft"), KeyNav::Collapse(1));
}

// ── row_key content hashing ─────────────────────────────────────────────

#[test]
fn row_key_changes_when_loading_flag_flips() {
    let base = FlatNode {
        key: "n".into(),
        label: "n".into(),
        is_branch: true,
        icon: None,
        depth: 0,
        expanded: false,
        children_loaded: false,
        loading: false,
    };
    let mut loading = base.clone();
    loading.loading = true;
    assert_ne!(
        row_key(&base),
        row_key(&loading),
        "loading flip must change the render key"
    );
}

#[test]
fn row_key_changes_when_expanded_or_label_changes() {
    let base = FlatNode {
        key: "n".into(),
        label: "n".into(),
        is_branch: true,
        icon: None,
        depth: 0,
        expanded: false,
        children_loaded: true,
        loading: false,
    };
    let mut expanded = base.clone();
    expanded.expanded = true;
    let mut relabeled = base.clone();
    relabeled.label = "n2".into();
    assert_ne!(row_key(&base), row_key(&expanded));
    assert_ne!(row_key(&base), row_key(&relabeled));
    assert_eq!(row_key(&base), row_key(&base.clone()), "stable for equal nodes");
}

// ── TreeNode / TreeChild builders ───────────────────────────────────────

#[test]
fn tree_node_builders() {
    let n = TreeNode::branch("k", "l").with_icon("📁").open();
    assert!(n.is_branch);
    assert_eq!(n.icon.as_deref(), Some("📁"));
    assert!(n.expanded);
    assert!(n.children.is_empty());

    let leaf = TreeNode::leaf("k", "l");
    assert!(!leaf.is_branch);

    let withkids = TreeNode::branch("k", "l").with_children(vec![TreeNode::leaf("c", "c")]);
    assert!(withkids.is_branch);
    assert_eq!(withkids.children.len(), 1);
}

#[test]
fn tree_child_builders() {
    assert!(!TreeChild::leaf("k", "l").is_branch);
    assert!(TreeChild::branch("k", "l").is_branch);
    assert_eq!(TreeChild::leaf("k", "l").with_icon("x").icon.as_deref(), Some("x"));
}

// ── should_spawn_load (double-load race guard) ──────────────────────────

#[test]
fn should_spawn_load_true_for_unloaded_not_loading_branch() {
    let nodes = build_flat(&[TreeNode::branch("lazy", "lazy")]);
    assert!(should_spawn_load(&nodes[0]));
}

#[test]
fn should_spawn_load_false_when_already_loading() {
    // Simulates: expand spawned a load (loading=true), then collapse+re-expand
    // observes the node before that load resolves — must not spawn a second one.
    let mut nodes = build_flat(&[TreeNode::branch("lazy", "lazy")]);
    nodes[0].loading = true;
    assert!(!should_spawn_load(&nodes[0]));
}

#[test]
fn should_spawn_load_false_when_already_loaded() {
    let nodes = build_flat(&[
        TreeNode::branch("eager", "eager").with_children(vec![TreeNode::leaf("c", "c")]),
    ]);
    assert!(!should_spawn_load(&nodes[0]), "eager branch already has children_loaded");
}

#[test]
fn should_spawn_load_false_for_leaf() {
    let nodes = build_flat(&[TreeNode::leaf("f", "f")]);
    assert!(!should_spawn_load(&nodes[0]));
}

#[test]
fn should_spawn_load_false_when_loaded_and_loading_both_set() {
    // Defensive: even a (shouldn't-happen) node with both flags set is not a
    // spawn candidate — children_loaded alone is enough to say "don't fetch".
    let mut nodes = build_flat(&[
        TreeNode::branch("eager", "eager").with_children(vec![TreeNode::leaf("c", "c")]),
    ]);
    nodes[0].loading = true;
    assert!(!should_spawn_load(&nodes[0]));
}

#[test]
fn flat_children_sets_depth_and_load_state() {
    let kids = flat_children(
        &[TreeChild::leaf("a", "a"), TreeChild::branch("b", "b")],
        3,
    );
    assert_eq!(kids[0].depth, 3);
    assert!(kids[0].children_loaded); // leaf
    assert!(!kids[1].children_loaded); // branch child not yet loaded
    assert!(!kids[1].expanded);
}
