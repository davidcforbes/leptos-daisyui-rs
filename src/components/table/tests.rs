use super::*;

// TableSize tests
#[test]
fn test_table_size_default() {
    let size = TableSize::default();
    assert_eq!(size.as_str(), "table-md");
}

#[test]
fn test_table_size_xs() {
    assert_eq!(TableSize::Xs.as_str(), "table-xs");
}

#[test]
fn test_table_size_sm() {
    assert_eq!(TableSize::Sm.as_str(), "table-sm");
}

#[test]
fn test_table_size_md() {
    assert_eq!(TableSize::Md.as_str(), "table-md");
}

#[test]
fn test_table_size_lg() {
    assert_eq!(TableSize::Lg.as_str(), "table-lg");
}

#[test]
fn test_table_size_xl() {
    assert_eq!(TableSize::Xl.as_str(), "table-xl");
}

#[test]
fn test_table_size_clone() {
    let s1 = TableSize::Lg;
    let s2 = s1.clone();
    assert_eq!(s1.as_str(), s2.as_str());
}

#[test]
fn test_table_size_debug() {
    let size = TableSize::Xs;
    assert!(format!("{:?}", size).contains("Xs"));
}

#[test]
fn test_all_table_sizes_return_valid_classes() {
    let variants = vec![
        (TableSize::Xs, "table-xs"),
        (TableSize::Sm, "table-sm"),
        (TableSize::Md, "table-md"),
        (TableSize::Lg, "table-lg"),
        (TableSize::Xl, "table-xl"),
    ];
    for (variant, expected) in variants {
        assert_eq!(variant.as_str(), expected);
    }
}

// TableViewport tests (op-ru6oi.12): the public horizontal-overflow frame.
#[test]
fn table_viewport_class_is_the_horizontal_overflow_contract() {
    assert!(
        TABLE_VIEWPORT_CLASS
            .split_ascii_whitespace()
            .any(|class| class == "overflow-x-auto"),
        "TableViewport stopped being a horizontal scroll viewport"
    );
}

/// Source contract, in the data_table `responsive_contract` style: the
/// component must (a) apply the shared class to its outer element and
/// (b) put the optional minimum width on the inner CONTENT div as an
/// inline `min-width` style -- a min-width on the scrolling element would
/// widen the viewport instead of making its content scrollable.
#[test]
fn table_viewport_applies_the_class_and_the_content_min_width() {
    let source = include_str!("viewport.rs");
    assert!(
        source.contains("merge_classes!(TABLE_VIEWPORT_CLASS, class)"),
        "TableViewport stopped applying the shared horizontal-scroll class"
    );
    assert!(
        source.contains("format!(\"min-width: {width}\")"),
        "TableViewport stopped carrying min_content_width as an inline min-width"
    );
}
