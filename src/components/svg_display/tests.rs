use super::*;

// SvgFit tests
#[test]
fn test_svg_fit_default() {
    let fit = SvgFit::default();
    assert_eq!(fit.as_str(), "");
}

#[test]
fn test_svg_fit_contain() {
    let fit = SvgFit::Contain;
    assert_eq!(fit.as_str(), "object-contain");
}

#[test]
fn test_svg_fit_cover() {
    let fit = SvgFit::Cover;
    assert_eq!(fit.as_str(), "object-cover");
}

#[test]
fn test_svg_fit_fill() {
    let fit = SvgFit::Fill;
    assert_eq!(fit.as_str(), "object-fill");
}

#[test]
fn test_svg_fit_scale_down() {
    let fit = SvgFit::ScaleDown;
    assert_eq!(fit.as_str(), "object-scale-down");
}

#[test]
fn test_svg_fit_none() {
    let fit = SvgFit::None;
    assert_eq!(fit.as_str(), "object-none");
}

#[test]
fn test_svg_fit_clone() {
    let fit1 = SvgFit::Contain;
    let fit2 = fit1.clone();
    assert_eq!(fit1.as_str(), fit2.as_str());
}

#[test]
fn test_svg_fit_debug() {
    let fit = SvgFit::ScaleDown;
    assert!(format!("{:?}", fit).contains("ScaleDown"));
}

// Comprehensive enum variant coverage test
#[test]
fn test_all_svg_fit_variants_return_valid_classes() {
    let fits = vec![
        (SvgFit::Default, ""),
        (SvgFit::Contain, "object-contain"),
        (SvgFit::Cover, "object-cover"),
        (SvgFit::Fill, "object-fill"),
        (SvgFit::ScaleDown, "object-scale-down"),
        (SvgFit::None, "object-none"),
    ];

    for (fit, expected) in fits {
        assert_eq!(fit.as_str(), expected);
    }
}
