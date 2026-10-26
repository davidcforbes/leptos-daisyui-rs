//! The inline-math rule that keeps prices as text (ldui-dtm3).
//!
//! pulldown-cmark (inside `editmark_core::render_html`) reads any `$…$` pair
//! as inline math, so `Pay $300-$500` arrives as
//! `<span class="math-inline">$300-$</span>500`, and once the span is
//! converted to MathML both dollar signs are gone. Pandoc tells a price from
//! a formula with three checks, and so does this module: the opening `$`
//! must be followed by a non-space character, the closing `$` must be
//! preceded by a non-space character, and the closing `$` must not be
//! followed by a digit.
//!
//! [`demote_currency_math`] applies the rule to the rendered HTML before it
//! reaches the DOM, in every build (the rule decides which `$…$` pairs are
//! math at all; the `math` cargo feature only decides whether the survivors
//! become MathML): a `.math-inline` span that fails it becomes its own
//! literal text with every `$` intact, so nothing is left for
//! [`super::math::render_math`] to convert. `$x^2$` still has its host.

const OPEN: &str = "<span class=\"math-inline\">$";
const CLOSE: &str = "$</span>";

/// Pandoc's inline-math rule over one `$…$` pair. `inner` is the text
/// between the dollars (HTML-escaped text is fine: only its first and last
/// characters are read) and `following` the character right after the
/// closing `$`, if there is one.
pub fn is_inline_math(inner: &str, following: Option<char>) -> bool {
    let (Some(first), Some(last)) = (inner.chars().next(), inner.chars().next_back()) else {
        return false;
    };
    !first.is_whitespace()
        && !last.is_whitespace()
        && !following.is_some_and(|c| c.is_ascii_digit())
}

/// Rewrite every `<span class="math-inline">$…$</span>` in `html` that fails
/// [`is_inline_math`] to its literal text. `html` is
/// `editmark_core::render_html` output, whose inline-math spans have exactly
/// this shape with escaped content, so the first `$</span>` after an opening
/// closes that span. Display math (`$$…$$`) is untouched.
pub fn demote_currency_math(html: &str) -> String {
    let mut out = String::with_capacity(html.len());
    let mut rest = html;
    while let Some(start) = rest.find(OPEN) {
        let after_open = start + OPEN.len();
        let Some(close_rel) = rest[after_open..].find(CLOSE) else {
            break;
        };
        let close = after_open + close_rel;
        let inner = &rest[after_open..close];
        let tail = &rest[close + CLOSE.len()..];
        out.push_str(&rest[..start]);
        if is_inline_math(inner, tail.chars().next()) {
            out.push_str(&rest[start..close + CLOSE.len()]);
        } else {
            out.push('$');
            out.push_str(inner);
            out.push('$');
        }
        rest = tail;
    }
    out.push_str(rest);
    out
}

#[cfg(test)]
mod tests {
    use super::{demote_currency_math, is_inline_math};

    #[test]
    fn pandoc_rule_reads_the_edges_of_the_pair() {
        assert!(is_inline_math("x^2", None));
        assert!(is_inline_math("x^2", Some('.')));
        assert!(is_inline_math("E=mc^2", Some(' ')));
        // A closing $ followed by a digit is a price, not math.
        assert!(!is_inline_math("300-", Some('5')));
        assert!(!is_inline_math("12,345.67 this week (", Some('1')));
        // An opening $ followed by whitespace never opens math.
        assert!(!is_inline_math(" x", None));
        // A closing $ preceded by whitespace never closes it.
        assert!(!is_inline_math("x ", None));
        assert!(!is_inline_math("", None));
    }

    #[test]
    fn prices_lose_their_math_host_and_keep_every_dollar() {
        let html = "<p>Pay <span class=\"math-inline\">$300-$</span>500</p>";
        assert_eq!(demote_currency_math(html), "<p>Pay $300-$500</p>");
        let html = "<p><span class=\"math-inline\">$300/$</span>500</p>";
        assert_eq!(demote_currency_math(html), "<p>$300/$500</p>");
        let html = "<p>Collected <span class=\"math-inline\">$12,345.67 this week ($</span>1,000 above plan)</p>";
        assert_eq!(
            demote_currency_math(html),
            "<p>Collected $12,345.67 this week ($1,000 above plan)</p>"
        );
    }

    #[test]
    fn formulas_keep_their_host_and_escaped_text_stays_escaped() {
        let html = "<p>Energy is <span class=\"math-inline\">$E=mc^2$</span>.</p>";
        assert_eq!(demote_currency_math(html), html);
        let html = "<p><span class=\"math-inline\">$a &lt; b$</span> and <span class=\"math-inline\">$ x$</span></p>";
        assert_eq!(
            demote_currency_math(html),
            "<p><span class=\"math-inline\">$a &lt; b$</span> and $ x$</p>"
        );
    }

    #[test]
    fn display_math_and_math_free_html_pass_through() {
        let html = "<div class=\"math-display\">$$\\int x$$</div><p>no math</p>";
        assert_eq!(demote_currency_math(html), html);
        assert_eq!(demote_currency_math(""), "");
    }
}
