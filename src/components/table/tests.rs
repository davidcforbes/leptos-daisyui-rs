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
