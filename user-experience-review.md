# User Experience Review: leptos-daisyui-rs Component Library

**Date:** 2026-03-22
**Scope:** Full library audit — UI consistency, UX gaps, code quality, test coverage
**Components audited:** 62 daisyUI wrappers + 33 utility/custom components (95 total)

---

## Executive Summary

The library provides a solid foundation with consistent component patterns and good type-safe styling via enums. However, there are significant gaps in **test coverage** (70% of components untested), **documentation typos**, **inconsistent derive traits** on style enums, and **missing `disabled` props** on form input components. These issues can be addressed systematically.

---

## 1. Documentation Bugs

### 1.1 Textarea doc references wrong element
**File:** `src/components/textarea/component.rs:16`
**Issue:** Doc comment says "References the table element" — should say "textarea element"
**Severity:** Low (cosmetic)

### 1.2 Breadcrumbs doc typo
**File:** `src/components/breadcrumbs/component.rs:17`
**Issue:** "Rederences" should be "References"
**Severity:** Low (cosmetic)

---

## 2. Missing `disabled` Prop on Form Inputs

Several form input components lack a `disabled` prop, while `Button` and `Select` correctly include one. This creates an inconsistent API — users expect form controls to be disableable.

| Component | Has `disabled`? | Has `color`/`size`? |
|-----------|:-:|:-:|
| Button | Yes | Yes |
| Select | Yes | Yes |
| Input | **No** | Yes |
| Textarea | **No** | Yes |
| Checkbox | **No** | Yes |
| Radio | **No** | Yes |
| Toggle | **No** | Yes |
| Range | **No** | Yes |
| FileInput | **No** | Yes |

**Recommendation:** Add `disabled: Signal<bool>` to all 7 missing components.

---

## 3. Inconsistent `PartialEq` Derives on Style Enums

Most style enums derive `Clone, Debug, Default` but NOT `PartialEq`. A few exceptions exist (e.g., `DividerDirection` has `PartialEq`). Without `PartialEq`, users cannot compare enum variants, which limits conditional logic.

**Components with style.rs missing PartialEq** (all 54 style.rs files):

The pattern `#[derive(Clone, Debug, Default)]` should be `#[derive(Clone, Debug, Default, PartialEq)]` for consistency and utility.

**Recommendation:** Add `PartialEq` to all style enums across all components.

---

## 4. Test Coverage Gaps

### Current State
- **25 components** have dedicated `tests.rs` files
- **3 components** have inline `#[cfg(test)]` modules (config_provider, data_table, gantt)
- **67 components** have **zero test coverage**

### Components with `style.rs` but NO tests (35 components)

These all have enum types that should be tested for correct `as_str()` class mapping:

| Component | Enum Types to Test |
|-----------|-------------------|
| divider | DividerColor, DividerDirection, DividerPlacement |
| dock | DockSize |
| drawer | DrawerPosition |
| dropdown | DropdownPosition, DropdownDirection |
| fab | FabLayout, FabPosition |
| field | FieldColor |
| file_input | FileInputStyle, FileInputColor, FileInputSize |
| footer | FooterDirection |
| indicator | IndicatorPosition, IndicatorColor |
| input | InputStyle, InputColor, InputSize |
| join | JoinDirection |
| kbd | KbdSize |
| link | LinkColor |
| loading | LoadingType, LoadingColor, LoadingSize |
| mask | MaskShape |
| menu | MenuSize |
| pagination | PaginationSize |
| progress | ProgressColor |
| radial_progress | RadialProgressColor |
| radio | RadioColor, RadioSize |
| range | RangeColor, RangeSize |
| rating | RatingSize |
| select | SelectStyle, SelectColor, SelectSize |
| slider | SliderColor, SliderSize |
| stack | StackDirection |
| status | StatusColor, StatusSize |
| steps | StepsDirection, StepColor |
| swap | SwapAnimation |
| tab | TabStyle, TabSize |
| table | TableSize |
| textarea | TextareaColor, TextareaSize |
| timeline | TimelineDirection, TimelineSize, TimelineColor |
| toast | ToastPosition, ToastColor |
| toggle | ToggleColor, ToggleSize |
| tooltip | TooltipPosition, TooltipColor |

### Components WITHOUT `style.rs` and NO tests (32 components)

These are simpler wrapper components. While they don't have enum types, they should at minimum have their module structure validated:

app_shell, auto_complete, base_theme_selector, breadcrumbs, calendar, color_picker, combobox, countdown, date_picker, diff, fieldset, filter, hero, hover_3d, hover_gallery, label, list, mockup_browser, mockup_code, mockup_phone, mockup_window, modal, navbar, skeleton, spin_button, stats, tag_picker, text_rotate, theme_controller, time_picker, upload_file, validator

---

## 5. Style Enum Consistency Audit

### 5.1 Default variant naming
All style enums consistently use `Default` as the variant name for the no-class state. This is good.

### 5.2 `as_str()` method
All enums use the same `as_str() -> &'static str` pattern. Consistent and correct.

### 5.3 Missing `Display` trait
No style enums implement `Display`. While not critical, it would improve ergonomics for logging and debugging.

---

## 6. Component API Consistency

### 6.1 `class` prop type
All components use `class: &'static str`. This is limiting but consistent. A future improvement could accept `Signal<String>` for dynamic classes, but this is a breaking API change and out of scope for this review.

### 6.2 `node_ref` availability
All components provide `node_ref` props. Sub-components (ModalBox, ModalAction, ModalBackdrop, BreadcrumbItem, SelectOption, etc.) also consistently provide `node_ref`. Good.

### 6.3 `merge_classes!` usage
Components correctly use `merge_classes!()` when they have a base daisyUI class. Components without a base class (Calendar wrapper, BreadcrumbItem `<li>`, CountdownValue `<span>`) correctly use `class=class` directly since there's no base class to merge.

### 6.4 Prop ordering
Props follow a generally consistent order: state/behavior props first, styling props, class, node_ref, children last. Minor variations exist but nothing problematic.

---

## 7. Actionable Fix Plan

### Priority 1: Quick Wins (Low Risk)
1. Fix 2 documentation typos
2. Add `PartialEq` derive to all style enums

### Priority 2: Consistency Fixes (Medium Risk)
3. Add `disabled` prop to 7 form input components

### Priority 3: Test Coverage (No Risk)
4. Add `tests.rs` for all 35 components with style enums
5. Each test file should cover: every variant's `as_str()`, Default, Clone, Debug, PartialEq

### Priority 4: Build Verification
6. Run `cargo test` to verify all new tests pass
7. Run `cargo clippy` to fix any linter warnings
8. Verify `cargo build` succeeds

---

## Appendix: Files Reviewed

- All 95 component directories under `src/components/`
- `src/utils/class_attribute.rs`
- `src/theme/` module (7 files)
- `demo/src/` (87 files)
- `demo/input.css`
- `Cargo.toml`
- All 25 existing `tests.rs` files
