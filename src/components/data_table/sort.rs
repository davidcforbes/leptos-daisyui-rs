//! Typed sorting for [`DataTable`](super::DataTable) columns.
//!
//! A table cell is a `String`, so comparing cells directly compares them
//! *lexicographically* — which is right for names and wrong for anything
//! numeric: `"$1,000" < "$900"` because `'1' < '9'`, and `"525" < "9"`.
//! Because [`Column::new`](super::Column::new) is sortable by default, a
//! consumer who puts money or durations in a table gets a silently wrong
//! sort. That is worse than no sort at all: the user believes they are
//! looking at the largest balances and they are not.
//!
//! A column therefore declares *how* its cells should be compared via
//! [`SortAs`], and this module turns the declaration into an [`Ordering`].
//! [`SortAs::Text`] is the default and is byte-for-byte the old behaviour,
//! so existing tables are unaffected.
//!
//! ## Missing values
//!
//! For [`SortAs::Number`] and [`SortAs::Date`], a cell that is absent, blank,
//! or unparseable is *missing*. Missing is not zero — an em dash means "not
//! measured", which is a different thing from a measured 0 and must not sort
//! between -1 and 1. Missing cells therefore sort **last in both directions**,
//! the way a spreadsheet puts blanks at the bottom, so a descending sort still
//! opens on the largest real value rather than on a wall of dashes.

use crate::components::data_table::types::{Column, SortOrder};
use std::cmp::Ordering;

/// How a column's cells should be compared when sorting.
///
/// Set on a [`Column`](super::Column) via
/// [`with_sort_as`](super::Column::with_sort_as). Defaults to [`Text`](Self::Text),
/// which preserves the plain lexicographic comparison.
///
/// ```
/// use leptos_daisyui_rs::components::{Column, SortAs};
///
/// // "$85" < "$900" < "$1,000" instead of the lexicographic "$1,000" < "$85".
/// let balance = Column::new("balance", "Balance").with_sort_as(SortAs::Number);
/// ```
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum SortAs {
    /// Compare cells as text (lexicographic, byte order). The default.
    #[default]
    Text,
    /// Compare cells as numbers, ignoring currency symbols, thousands
    /// separators, percent signs, and surrounding whitespace. See
    /// [`parse_number`].
    Number,
    /// Compare cells as dates. See [`parse_date`] for the accepted formats.
    Date,
}

/// Parse a formatted number out of a cell string, returning `None` when the
/// cell holds no number (blank, an em dash, `"N/A"`, …).
///
/// Everything that is not a digit, a decimal point, or a leading sign is
/// discarded, so the usual display formats all parse to the value they show:
///
/// | Cell         | Value     |
/// |--------------|-----------|
/// | `"$1,000.50"`| `1000.5`  |
/// | `"9%"`       | `9.0`     |
/// | `"-12"`      | `-12.0`   |
/// | `"(1,234)"`  | `-1234.0` |
/// | `"—"`        | `None`    |
///
/// Parentheses are read as an accounting negative. A leading ASCII `-` or
/// Unicode minus (`−`) also negates. The result is always finite: a cell that
/// parses to infinity or NaN is reported missing rather than sorted to an end.
pub fn parse_number(cell: &str) -> Option<f64> {
    let trimmed = cell.trim();
    if trimmed.is_empty() {
        return None;
    }

    // Accounting negative: "(1,234)" means -1234.
    let (body, mut negative) = match trimmed.strip_prefix('(').and_then(|s| s.strip_suffix(')')) {
        Some(inner) => (inner, true),
        None => (trimmed, false),
    };

    let mut digits = String::with_capacity(body.len());
    let mut seen_digit = false;
    for ch in body.chars() {
        match ch {
            '0'..='9' => {
                digits.push(ch);
                seen_digit = true;
            }
            // A decimal point only counts before we would create a second one;
            // "1.2.3" is not a number and must fall through to a parse failure.
            '.' => digits.push(ch),
            // A sign is only a sign in front of the number. A trailing "-" or
            // an internal one (a range like "3-5") is not.
            '-' | '\u{2212}' if !seen_digit && digits.is_empty() => negative = true,
            // Currency symbols, thousands separators, "%", spaces, stray text.
            _ => {}
        }
    }

    if !seen_digit {
        return None;
    }

    let magnitude: f64 = digits.parse().ok()?;
    if !magnitude.is_finite() {
        return None;
    }
    Some(if negative { -magnitude } else { magnitude })
}

/// Parse a date out of a cell string into a monotonically comparable key,
/// returning `None` when the cell holds no recognisable date.
///
/// Accepted formats:
/// - ISO 8601 date or date-time: `2026-07-13`, `2026-07-13T09:30`,
///   `2026-07-13 09:30:15` (anything after the seconds is ignored).
/// - US slash form: `7/13/2026` — month first, matching how these columns are
///   formatted for display in en-US locales.
///
/// The returned `f64` is an ordering key only (a packed `YYYYMMDDhhmmss`), not
/// a timestamp — compare it, do not do arithmetic on it. It is exactly
/// representable in `f64`.
pub fn parse_date(cell: &str) -> Option<f64> {
    let trimmed = cell.trim();
    if trimmed.is_empty() {
        return None;
    }

    // Split the date part from an optional time part.
    let (date_part, time_part) = match trimmed.split_once(['T', ' ']) {
        Some((d, t)) => (d, Some(t)),
        None => (trimmed, None),
    };

    let (year, month, day) = if date_part.contains('-') {
        let mut fields = date_part.split('-');
        let y = fields.next()?.parse::<u32>().ok()?;
        let m = fields.next()?.parse::<u32>().ok()?;
        let d = fields.next()?.parse::<u32>().ok()?;
        if fields.next().is_some() {
            return None;
        }
        (y, m, d)
    } else if date_part.contains('/') {
        let mut fields = date_part.split('/');
        let m = fields.next()?.parse::<u32>().ok()?;
        let d = fields.next()?.parse::<u32>().ok()?;
        let y = fields.next()?.parse::<u32>().ok()?;
        if fields.next().is_some() {
            return None;
        }
        (y, m, d)
    } else {
        return None;
    };

    if !(1..=12).contains(&month) || !(1..=31).contains(&day) || year > 9999 {
        return None;
    }

    // Time is optional; a malformed time degrades to midnight rather than
    // discarding an otherwise good date.
    let (hour, minute, second) = time_part.map(parse_time).unwrap_or((0, 0, 0));

    let key = year as u64 * 10_000_000_000
        + month as u64 * 100_000_000
        + day as u64 * 1_000_000
        + hour as u64 * 10_000
        + minute as u64 * 100
        + second as u64;
    Some(key as f64)
}

/// Best-effort `hh[:mm[:ss]]` parse. Out-of-range or unparseable fields are
/// dropped to zero so a good date is never thrown away over a bad clock.
fn parse_time(time: &str) -> (u32, u32, u32) {
    let mut fields = time.trim().split(':');
    let hour = field_in_range(fields.next(), 23);
    let minute = field_in_range(fields.next(), 59);
    // Trailing fraction / timezone ("15.250Z") is not part of the ordering key.
    let second = field_in_range(
        fields
            .next()
            .map(|s| s.split(['.', '+', 'Z', 'z']).next().unwrap_or("")),
        59,
    );
    (hour, minute, second)
}

fn field_in_range(field: Option<&str>, max: u32) -> u32 {
    field
        .and_then(|s| s.trim().parse::<u32>().ok())
        .filter(|v| *v <= max)
        .unwrap_or(0)
}

/// The [`SortAs`] declared by the column with this id, or the default
/// ([`SortAs::Text`]) when no such column exists — a sort on an unknown column
/// must degrade to the old text comparison, never panic.
pub fn column_sort_as(columns: &[Column], col_id: &str) -> SortAs {
    columns
        .iter()
        .find(|c| c.id == col_id)
        .map(|c| c.sort_as)
        .unwrap_or_default()
}

/// Compare two raw cell strings for a column declared `sort_as`, honouring
/// `order`.
///
/// For [`SortAs::Text`] this is the plain lexicographic comparison, reversed
/// for [`SortOrder::Desc`]. For [`SortAs::Number`] and [`SortAs::Date`] the
/// cells are parsed first, and missing values sort last in *both* directions
/// (see the module docs).
pub fn compare_cells(a: &str, b: &str, sort_as: SortAs, order: SortOrder) -> Ordering {
    match sort_as {
        SortAs::Text => match order {
            SortOrder::Asc => a.cmp(b),
            SortOrder::Desc => b.cmp(a),
        },
        SortAs::Number => compare_parsed(parse_number(a), parse_number(b), order),
        SortAs::Date => compare_parsed(parse_date(a), parse_date(b), order),
    }
}

/// Order two parsed keys, keeping missing values pinned to the bottom in both
/// sort directions. `f64::total_cmp` gives a total order, so the comparator
/// stays a valid `Ord` even if a cell somehow yields a NaN.
fn compare_parsed(a: Option<f64>, b: Option<f64>, order: SortOrder) -> Ordering {
    match (a, b) {
        (Some(a), Some(b)) => match order {
            SortOrder::Asc => a.total_cmp(&b),
            SortOrder::Desc => b.total_cmp(&a),
        },
        (Some(_), None) => Ordering::Less, // present before missing
        (None, Some(_)) => Ordering::Greater, // missing after present
        (None, None) => Ordering::Equal,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// Sort a column of cells the way the DataTable memo does, so the tests
    /// assert on the user-visible row order rather than on pairwise orderings.
    fn sorted(cells: &[&str], sort_as: SortAs, order: SortOrder) -> Vec<String> {
        let mut out: Vec<String> = cells.iter().map(|s| s.to_string()).collect();
        out.sort_by(|a, b| compare_cells(a, b, sort_as, order));
        out
    }

    // ── parse_number ──

    #[test]
    fn parse_number_plain_integer() {
        assert_eq!(parse_number("525"), Some(525.0));
    }

    #[test]
    fn parse_number_strips_currency_and_separators() {
        assert_eq!(parse_number("$1,000.50"), Some(1000.5));
        assert_eq!(parse_number("€2 000"), Some(2000.0));
        assert_eq!(parse_number("£85"), Some(85.0));
    }

    #[test]
    fn parse_number_strips_percent() {
        assert_eq!(parse_number("9%"), Some(9.0));
        assert_eq!(parse_number("10%"), Some(10.0));
    }

    #[test]
    fn parse_number_handles_negatives() {
        assert_eq!(parse_number("-12"), Some(-12.0));
        assert_eq!(parse_number("\u{2212}12"), Some(-12.0)); // Unicode minus
        assert_eq!(parse_number("-$1,200.25"), Some(-1200.25));
    }

    #[test]
    fn parse_number_reads_parentheses_as_accounting_negative() {
        assert_eq!(parse_number("(1,234)"), Some(-1234.0));
        assert_eq!(parse_number("($99.50)"), Some(-99.5));
    }

    #[test]
    fn parse_number_returns_none_for_missing_cells() {
        assert_eq!(parse_number(""), None);
        assert_eq!(parse_number("   "), None);
        assert_eq!(parse_number("\u{2014}"), None); // em dash
        assert_eq!(parse_number("N/A"), None);
        assert_eq!(parse_number("--"), None);
    }

    #[test]
    fn parse_number_returns_none_for_malformed_numbers() {
        assert_eq!(parse_number("1.2.3"), None);
        assert_eq!(parse_number("$"), None);
    }

    #[test]
    fn parse_number_zero_is_a_value_not_a_missing_cell() {
        assert_eq!(parse_number("0"), Some(0.0));
        assert_eq!(parse_number("$0.00"), Some(0.0));
    }

    // ── parse_date ──

    #[test]
    fn parse_date_iso_dates_order_correctly() {
        let a = parse_date("2026-07-13").unwrap();
        let b = parse_date("2026-11-02").unwrap();
        let c = parse_date("2027-01-01").unwrap();
        assert!(a < b && b < c);
    }

    #[test]
    fn parse_date_iso_datetime_orders_within_a_day() {
        let morning = parse_date("2026-07-13T09:30:00").unwrap();
        let evening = parse_date("2026-07-13 21:05:15").unwrap();
        assert!(morning < evening);
        assert!(parse_date("2026-07-13").unwrap() <= morning);
    }

    #[test]
    fn parse_date_us_slash_form_is_month_first() {
        // 7/13/2026 is July 13th; if it parsed day-first it would be invalid.
        assert_eq!(parse_date("7/13/2026"), parse_date("2026-07-13"));
    }

    #[test]
    fn parse_date_returns_none_for_missing_or_malformed_cells() {
        assert_eq!(parse_date(""), None);
        assert_eq!(parse_date("\u{2014}"), None);
        assert_eq!(parse_date("not a date"), None);
        assert_eq!(parse_date("2026-13-01"), None); // month 13
        assert_eq!(parse_date("2026-07-32"), None); // day 32
        assert_eq!(parse_date("2026-07-13-01"), None); // too many fields
    }

    #[test]
    fn parse_date_key_is_packed_yyyymmddhhmmss_and_exact_in_f64() {
        let key = parse_date("2026-07-13T23:59:59").unwrap();
        assert_eq!(key, 20_260_713_235_959.0);
    }

    // ── Acceptance criteria from the bug report ──

    #[test]
    fn money_column_sorts_by_value_not_by_first_digit() {
        assert_eq!(
            sorted(&["$900", "$1,000", "$85"], SortAs::Number, SortOrder::Asc),
            ["$85", "$900", "$1,000"]
        );
    }

    #[test]
    fn duration_column_sorts_by_value() {
        assert_eq!(
            sorted(&["9", "525", "10"], SortAs::Number, SortOrder::Asc),
            ["9", "10", "525"]
        );
    }

    #[test]
    fn text_columns_keep_the_old_lexicographic_order() {
        assert_eq!(
            sorted(&["$900", "$1,000", "$85"], SortAs::Text, SortOrder::Asc),
            ["$1,000", "$85", "$900"]
        );
        assert_eq!(
            sorted(&["Charlie", "alice", "Bob"], SortAs::Text, SortOrder::Asc),
            ["Bob", "Charlie", "alice"]
        );
    }

    #[test]
    fn text_is_the_default_sort() {
        assert_eq!(SortAs::default(), SortAs::Text);
    }

    #[test]
    fn missing_cells_are_not_conflated_with_zero() {
        // The em dash means "not measured" and must not land between -5 and 3.
        assert_eq!(
            sorted(
                &["3", "\u{2014}", "-5", "0"],
                SortAs::Number,
                SortOrder::Asc
            ),
            ["-5", "0", "3", "\u{2014}"]
        );
    }

    #[test]
    fn missing_cells_sort_last_in_both_directions() {
        assert_eq!(
            sorted(&["$85", "", "$900"], SortAs::Number, SortOrder::Asc),
            ["$85", "$900", ""]
        );
        assert_eq!(
            sorted(&["$85", "", "$900"], SortAs::Number, SortOrder::Desc),
            ["$900", "$85", ""]
        );
    }

    #[test]
    fn missing_dates_sort_last_in_both_directions() {
        assert_eq!(
            sorted(
                &["2026-11-02", "\u{2014}", "2026-07-13"],
                SortAs::Date,
                SortOrder::Asc
            ),
            ["2026-07-13", "2026-11-02", "\u{2014}"]
        );
        assert_eq!(
            sorted(
                &["2026-11-02", "\u{2014}", "2026-07-13"],
                SortAs::Date,
                SortOrder::Desc
            ),
            ["2026-11-02", "2026-07-13", "\u{2014}"]
        );
    }

    #[test]
    fn descending_number_sort_reverses_values() {
        assert_eq!(
            sorted(&["9", "525", "10"], SortAs::Number, SortOrder::Desc),
            ["525", "10", "9"]
        );
    }

    // ── column_sort_as ──

    #[test]
    fn column_sort_as_finds_the_declared_type() {
        let columns = vec![
            Column::new("name", "Name"),
            Column::new("balance", "Balance").with_sort_as(SortAs::Number),
            Column::new("opened", "Opened").with_sort_as(SortAs::Date),
        ];
        assert_eq!(column_sort_as(&columns, "name"), SortAs::Text);
        assert_eq!(column_sort_as(&columns, "balance"), SortAs::Number);
        assert_eq!(column_sort_as(&columns, "opened"), SortAs::Date);
    }

    #[test]
    fn column_sort_as_defaults_to_text_for_an_unknown_column() {
        let columns = vec![Column::new("name", "Name").with_sort_as(SortAs::Number)];
        assert_eq!(column_sort_as(&columns, "ghost"), SortAs::Text);
        assert_eq!(column_sort_as(&[], "name"), SortAs::Text);
    }

    // ── comparator sanity ──

    #[test]
    fn compare_cells_is_reflexive_and_symmetric() {
        assert_eq!(
            compare_cells("$100", "$100", SortAs::Number, SortOrder::Asc),
            Ordering::Equal
        );
        assert_eq!(
            compare_cells("$100", "$200", SortAs::Number, SortOrder::Asc),
            Ordering::Less
        );
        assert_eq!(
            compare_cells("$200", "$100", SortAs::Number, SortOrder::Asc),
            Ordering::Greater
        );
    }

    #[test]
    fn two_missing_cells_compare_equal_so_their_relative_order_is_stable() {
        assert_eq!(
            compare_cells("\u{2014}", "", SortAs::Number, SortOrder::Asc),
            Ordering::Equal
        );
        assert_eq!(
            compare_cells("\u{2014}", "", SortAs::Number, SortOrder::Desc),
            Ordering::Equal
        );
    }
}
