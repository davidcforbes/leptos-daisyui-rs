//! Lexical pre-check run before every math-core conversion (ldui-r0co).
//!
//! math-core parses recursively, and on wasm's 1 MiB stack a deep enough
//! formula overflows it: measured at 451 nested `\frac`, 797 nested
//! superscripts and 849 nested braces. On wasm that is a trap, not a
//! panic, and it leaves the whole app unusable. So nothing reaches the
//! converter until this scan has bounded its nesting.
//!
//! The scan counts, at every point, open braces + open `\left` + open
//! `\begin` + the length of the current run of back-to-back control words
//! (`\sqrt\sqrt\sqrt x` nests without braces).
//!
//! How deep is safe depends on the build profile, because an unoptimised
//! build spends far more stack per level. Measured on a 1 MiB stack
//! (math-core 0.9.0, 2026-10-23, native x86-64; the wasm release figure
//! from the research build was 451 `\frac`):
//!
//! | Construct | debug: deepest that converts | release |
//! |---|---|---|
//! | nested `\frac{1}{…}` | 18 | 335 |
//! | nested braces / superscripts | 34 / 32 | 591 / 491 |
//! | `\left(` / `\begin{matrix}` | 34 / 34 | 592 / 550 |
//! | `\sqrt\sqrt…` chain | 38 | 793 |
//!
//! [`MAX_MATH_DEPTH`] is therefore 64 in release builds (five times below
//! the lowest release figure) and 12 in debug builds such as `trunk serve`
//! without `--release`, leaving the rest of the stack for the frames Leptos
//! already has when math renders. Everyday formulas stay under 8.
//!
//! Macro definitions are refused outright: a `\newcommand` whose body is 60
//! levels deep, applied to itself, nests far deeper than its source shows,
//! and no lexical bound can see through that.

/// Deepest nesting a formula may have: 64 in release builds, 12 in debug
/// builds (see the module docs for the measurements).
pub const MAX_MATH_DEPTH: usize = if cfg!(debug_assertions) { 12 } else { 64 };

/// Longest formula, in bytes, that is converted.
pub const MAX_MATH_BYTES: usize = 8192;

/// Commands that define macros, refused because expansion hides depth.
const MACRO_DEFINITIONS: &[&str] = &[
    "def",
    "gdef",
    "let",
    "global",
    "newcommand",
    "renewcommand",
    "providecommand",
];

/// Why a formula was not converted.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum GuardError {
    /// Longer than [`MAX_MATH_BYTES`].
    TooLong(usize),
    /// Nested deeper than [`MAX_MATH_DEPTH`].
    TooDeep,
    /// Uses a macro definition command (the name, without the backslash).
    MacroDefinition(String),
}

impl GuardError {
    /// A message for the formula's tooltip.
    pub fn message(&self) -> String {
        match self {
            GuardError::TooLong(n) => {
                format!("Formula is too long to render ({n} bytes; the limit is {MAX_MATH_BYTES}).")
            }
            GuardError::TooDeep => format!(
                "Formula is nested too deeply to render (the limit is {MAX_MATH_DEPTH} levels)."
            ),
            GuardError::MacroDefinition(name) => {
                format!("Macro definitions (\\{name}) are not supported here.")
            }
        }
    }
}

/// Accept `latex` for conversion, or say why not.
pub fn check_formula(latex: &str) -> Result<(), GuardError> {
    if latex.len() > MAX_MATH_BYTES {
        return Err(GuardError::TooLong(latex.len()));
    }
    let bytes = latex.as_bytes();
    let mut braces = 0usize;
    let mut lefts = 0usize;
    let mut envs = 0usize;
    let mut chain = 0usize;
    let mut i = 0;
    while i < bytes.len() {
        let b = bytes[i];
        match b {
            b'\\' => {
                let start = i + 1;
                let mut end = start;
                while end < bytes.len() && bytes[end].is_ascii_alphabetic() {
                    end += 1;
                }
                if end == start {
                    // Control symbol (`\{`, `\\`, `\,`): one character, no nesting.
                    chain = 0;
                    i = (start + 1).min(bytes.len());
                    // Skip a whole UTF-8 character after the backslash.
                    while i < bytes.len() && !latex.is_char_boundary(i) {
                        i += 1;
                    }
                    continue;
                }
                let name = &latex[start..end];
                if MACRO_DEFINITIONS.contains(&name) {
                    return Err(GuardError::MacroDefinition(name.to_string()));
                }
                match name {
                    "left" | "begin" => {
                        if name == "left" {
                            lefts += 1;
                        } else {
                            envs += 1;
                        }
                        chain = 0;
                    }
                    "right" => {
                        lefts = lefts.saturating_sub(1);
                        chain = 0;
                    }
                    "end" => {
                        envs = envs.saturating_sub(1);
                        chain = 0;
                    }
                    _ => chain += 1,
                }
                i = end;
            }
            b'{' => {
                braces += 1;
                chain = 0;
                i += 1;
            }
            b'}' => {
                braces = braces.saturating_sub(1);
                chain = 0;
                i += 1;
            }
            b' ' | b'\t' | b'\n' | b'\r' => i += 1,
            _ => {
                chain = 0;
                i += 1;
            }
        }
        if braces + lefts + envs + chain > MAX_MATH_DEPTH {
            return Err(GuardError::TooDeep);
        }
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    fn nested_fracs(n: usize) -> String {
        let mut s = String::new();
        for _ in 0..n {
            s.push_str("\\frac{1}{");
        }
        s.push('x');
        for _ in 0..n {
            s.push('}');
        }
        s
    }

    #[test]
    fn everyday_formulas_pass() {
        for f in [
            "E=mc^2",
            "\\int_0^1 x^2\\,dx = \\frac{1}{3}",
            "\\begin{pmatrix} a & b \\\\ c & d \\end{pmatrix}",
            "\\left( \\sum_{i=1}^{n} i \\right)^2",
            "\\sqrt[3]{\\frac{a}{b}}",
            "f(x) = \\begin{cases} 1 & x > 0 \\\\ 0 & \\text{otherwise} \\end{cases}",
            "\\mathbb{R} \\to \\mathcal{C}",
            "\\{ x \\mid x > 0 \\}",
        ] {
            assert_eq!(check_formula(f), Ok(()), "{f}");
        }
    }

    #[test]
    fn depth_is_capped_at_the_limit_not_before() {
        assert_eq!(check_formula(&nested_fracs(MAX_MATH_DEPTH - 1)), Ok(()));
        assert_eq!(
            check_formula(&nested_fracs(MAX_MATH_DEPTH + 1)),
            Err(GuardError::TooDeep)
        );
    }

    #[test]
    fn hostile_depths_are_refused() {
        assert_eq!(check_formula(&nested_fracs(451)), Err(GuardError::TooDeep));
        let braces = format!("{}x{}", "{".repeat(20_000), "}".repeat(20_000));
        assert_eq!(check_formula(&braces), Err(GuardError::TooLong(40_001)));
        let braces = format!("{}x{}", "{".repeat(100), "}".repeat(100));
        assert_eq!(check_formula(&braces), Err(GuardError::TooDeep));
        let lefts = "\\left(".repeat(100);
        assert_eq!(check_formula(&lefts), Err(GuardError::TooDeep));
        let envs = "\\begin{matrix}".repeat(70);
        assert_eq!(check_formula(&envs), Err(GuardError::TooDeep));
    }

    #[test]
    fn unbraced_command_chains_count_as_nesting() {
        let chain = format!("{}x", "\\sqrt".repeat(MAX_MATH_DEPTH + 1));
        assert_eq!(check_formula(&chain), Err(GuardError::TooDeep));
        let spaced = format!("{}x", "\\sqrt ".repeat(MAX_MATH_DEPTH + 1));
        assert_eq!(check_formula(&spaced), Err(GuardError::TooDeep));
        // A long flat run separated by ordinary characters is not nesting.
        let flat = "\\alpha+".repeat(500);
        assert_eq!(check_formula(&flat), Ok(()));
    }

    #[test]
    fn macro_definitions_are_refused() {
        for (f, name) in [
            ("\\newcommand{\\R}{\\mathbb{R}} \\R", "newcommand"),
            ("\\renewcommand\\x{y}", "renewcommand"),
            ("\\def\\a{\\a\\a}\\a", "def"),
            ("\\let\\b=\\a", "let"),
        ] {
            assert_eq!(
                check_formula(f),
                Err(GuardError::MacroDefinition(name.into())),
                "{f}"
            );
        }
    }

    #[test]
    fn length_is_capped() {
        let long = "x".repeat(MAX_MATH_BYTES + 1);
        assert_eq!(
            check_formula(&long),
            Err(GuardError::TooLong(MAX_MATH_BYTES + 1))
        );
    }

    #[test]
    fn control_symbols_before_multibyte_text_do_not_split_characters() {
        assert_eq!(check_formula("\\é + \\ü"), Ok(()));
    }
}
