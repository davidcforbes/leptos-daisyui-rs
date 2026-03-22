use super::*;

// DockSize tests
#[test]
fn test_dock_size_default() {
    let val = DockSize::default();
    assert_eq!(val.as_str(), "");
}

#[test]
fn test_dock_size_xs() {
    let val = DockSize::Xs;
    assert_eq!(val.as_str(), "dock-xs");
}

#[test]
fn test_dock_size_sm() {
    let val = DockSize::Sm;
    assert_eq!(val.as_str(), "dock-sm");
}

#[test]
fn test_dock_size_md() {
    let val = DockSize::Md;
    assert_eq!(val.as_str(), "dock-md");
}

#[test]
fn test_dock_size_lg() {
    let val = DockSize::Lg;
    assert_eq!(val.as_str(), "dock-lg");
}

#[test]
fn test_dock_size_xl() {
    let val = DockSize::Xl;
    assert_eq!(val.as_str(), "dock-xl");
}

#[test]
fn test_dock_size_clone() {
    let v1 = DockSize::Lg;
    let v2 = v1.clone();
    assert_eq!(v1.as_str(), v2.as_str());
}

#[test]
fn test_dock_size_debug() {
    let val = DockSize::Xl;
    assert!(format!("{:?}", val).contains("Xl"));
}

// Comprehensive coverage test
#[test]
fn test_all_dock_sizes_return_valid_classes() {
    let variants = vec![
        (DockSize::Default, ""),
        (DockSize::Xs, "dock-xs"),
        (DockSize::Sm, "dock-sm"),
        (DockSize::Md, "dock-md"),
        (DockSize::Lg, "dock-lg"),
        (DockSize::Xl, "dock-xl"),
    ];
    for (variant, expected) in variants {
        assert_eq!(variant.as_str(), expected);
    }
}
