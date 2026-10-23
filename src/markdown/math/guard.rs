//! Lexical pre-check run before every math-core conversion (ldui-r0co).
//!
//! math-core parses recursively, and on wasm's 1 MiB stack a deep enough
//! formula overflows it: measured at 451 nested `\frac`, 797 nested
//! superscripts and 849 nested braces. On wasm that is a trap, not a
//! panic, and it leaves the whole app unusable. So nothing reaches the
//! converter until this scan has bounded its nesting.
//!
//! The scan simulates how math-core nests, on one stack: every `{`, `[`,
//! `\left`, `\begin` and `\begingroup` opens a level, and so does every
//! command still waiting for arguments. How many arguments each command
//! takes comes from math-core's own command table ([`super::arity`],
//! generated from the pinned version), because unbraced arguments nest
//! without a single brace: `\frac1\frac1\frac1x`, `\sqrt[\sqrt[…]]`,
//! `\frac\left(x\right)\frac…`. A command whose arguments are all in
//! closes its level and counts as one argument of the level below. Where
//! the simulation could be unsure it over-counts (a stray `[`, a prime or
//! `&` never completes an argument), so it can refuse a strange formula but
//! never under-count a deep one.
//!
//! How deep is safe depends on the build profile, because an unoptimised
//! build spends far more stack per level. Measured on a 1 MiB stack
//! (math-core 0.9.0, 2026-10-23, native x86-64), as the deepest that still
//! converts, in levels and in this scan's stack entries:
//!
//! | Pattern | entries/level | debug levels (entries) | release levels (entries) |
//! |---|---|---|---|
//! | `\frac{1}{…}` | 2 | 18 (36) | 335 (670) |
//! | `{…}` / `\left(` / `\begin{matrix}` | 1 | 35 / 35 / 34 | 591 / 591 / 550 |
//! | `\sqrt\sqrt…` / `\frac1\frac1…` | 1 | 38 / 37 | 793 / 773 |
//! | `\frac\left(x\right)…` / `\genfrac…` | 1 | 36 / 36 | 771 / 771 |
//! | `x^{…}` / `\sqrt[…]` / `\textcolor{c}{…}` | 2 | 32 / 34 / 17 (64 / 68 / 34) | 491 / 591 / 285 |
//!
//! So one entry costs at most about 31 KB of stack in debug and 1.9 KB in
//! release, and the fewest entries that overflow are 34 and 550.
//! [`MAX_MATH_DEPTH`] is 20 entries in debug builds such as `trunk serve`
//! without `--release` (about 620 KB, leaving the rest for the frames
//! Leptos already has when math renders) and 64 in release builds (about
//! 122 KB). Everyday formulas stay under 10 entries.
//!
//! Macro definitions are refused outright: a `\newcommand` whose body is 60
//! levels deep, applied to itself, nests far deeper than its source shows,
//! and no lexical bound can see through that.

/// Most open levels (stack entries) a formula may reach: 64 in release
/// builds, 20 in debug builds (see the module docs for the measurements).
pub const MAX_MATH_DEPTH: usize = if cfg!(debug_assertions) { 20 } else { 64 };

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

/// One open level of the simulation.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum Level {
    Brace,
    Bracket,
    Left,
    Env,
    Group,
    /// A command still waiting for `left` arguments. A finished command
    /// counts as one argument of the level below, except a script (`^`,
    /// `_`), which attaches to a base that was already counted.
    Args {
        left: u8,
        counts: bool,
    },
}

struct Scan<'a> {
    s: &'a str,
    i: usize,
    stack: Vec<Level>,
}

impl Scan<'_> {
    fn push(&mut self, level: Level) -> Result<(), GuardError> {
        self.stack.push(level);
        if self.stack.len() > MAX_MATH_DEPTH {
            Err(GuardError::TooDeep)
        } else {
            Ok(())
        }
    }

    /// One argument-sized unit finished: feed it to the waiting command.
    fn argument_done(&mut self) {
        while let Some(Level::Args { left, counts }) = self.stack.last_mut() {
            *left = left.saturating_sub(1);
            if *left > 0 {
                return;
            }
            let counts = *counts;
            self.stack.pop();
            if !counts {
                return;
            }
        }
    }

    /// Close the innermost `level`; false when none is open.
    fn close(&mut self, level: Level) -> bool {
        match self.stack.iter().rposition(|l| *l == level) {
            Some(at) => {
                self.stack.truncate(at);
                true
            }
            None => false,
        }
    }

    fn skip_ws(&mut self) {
        let rest = &self.s[self.i..];
        self.i += rest.len() - rest.trim_start().len();
    }

    /// Skip one token: a control word, a control symbol or a character.
    fn skip_token(&mut self) {
        self.skip_ws();
        let rest = &self.s[self.i..];
        let Some(c) = rest.chars().next() else { return };
        if c == '\\' {
            let word = rest[1..]
                .bytes()
                .take_while(u8::is_ascii_alphabetic)
                .count();
            self.i += 1 + if word > 0 {
                word
            } else {
                rest[1..].chars().next().map_or(0, char::len_utf8)
            };
        } else {
            self.i += c.len_utf8();
        }
    }

    /// Skip an environment name `{…}` after `\begin` / `\end`.
    fn skip_env_name(&mut self) {
        self.skip_ws();
        if self.s[self.i..].starts_with('{') {
            self.i += self.s[self.i..]
                .find('}')
                .map_or(self.s.len() - self.i, |e| e + 1);
        }
    }

    fn run(&mut self) -> Result<(), GuardError> {
        while let Some(c) = self.s[self.i..].chars().next() {
            self.i += c.len_utf8();
            match c {
                c if c.is_whitespace() => {}
                '\\' => self.control()?,
                '{' => self.push(Level::Brace)?,
                '}' => {
                    self.close(Level::Brace);
                    self.argument_done();
                }
                '[' => self.push(Level::Bracket)?,
                ']' => {
                    // An optional argument closes; a lone `]` is a character.
                    if !self.close(Level::Bracket) {
                        self.argument_done();
                    }
                }
                '^' | '_' => self.push(Level::Args {
                    left: 1,
                    counts: false,
                })?,
                // Primes and column separators never finish an argument.
                '\'' | '&' => {}
                _ => self.argument_done(),
            }
        }
        Ok(())
    }

    fn control(&mut self) -> Result<(), GuardError> {
        let rest = &self.s[self.i..];
        let len = rest.bytes().take_while(u8::is_ascii_alphabetic).count();
        if len == 0 {
            // Control symbol (`\{`, `\\`, `\,`): one token.
            self.i += rest.chars().next().map_or(0, char::len_utf8);
            self.argument_done();
            return Ok(());
        }
        let name = &rest[..len];
        self.i += len;
        if MACRO_DEFINITIONS.contains(&name) {
            return Err(GuardError::MacroDefinition(name.to_string()));
        }
        match name {
            "left" => {
                self.push(Level::Left)?;
                self.skip_token();
            }
            "right" => {
                self.close(Level::Left);
                self.skip_token();
                self.argument_done();
            }
            "middle" => self.skip_token(),
            "begin" => {
                self.push(Level::Env)?;
                self.skip_env_name();
            }
            "end" => {
                self.close(Level::Env);
                self.skip_env_name();
                self.argument_done();
            }
            "begingroup" => self.push(Level::Group)?,
            "endgroup" => {
                self.close(Level::Group);
                self.argument_done();
            }
            _ => match arguments(name) {
                0 => self.argument_done(),
                n => self.push(Level::Args {
                    left: n,
                    counts: true,
                })?,
            },
        }
        Ok(())
    }
}

/// How many arguments `name` takes (0 for symbols and unknown words).
fn arguments(name: &str) -> u8 {
    super::arity::ARGUMENT_COMMANDS
        .binary_search_by(|(n, _)| (*n).cmp(name))
        .map_or(0, |at| super::arity::ARGUMENT_COMMANDS[at].1)
}

/// Accept `latex` for conversion, or say why not.
pub fn check_formula(latex: &str) -> Result<(), GuardError> {
    if latex.len() > MAX_MATH_BYTES {
        return Err(GuardError::TooLong(latex.len()));
    }
    Scan {
        s: latex,
        i: 0,
        stack: Vec::new(),
    }
    .run()
}

#[cfg(test)]
mod tests {
    use super::*;

    /// Hostile nesting patterns, each `n` levels deep.
    fn patterns(n: usize) -> Vec<(&'static str, String)> {
        vec![
            (
                "frac",
                format!("{}x{}", r"\frac{1}{".repeat(n), "}".repeat(n)),
            ),
            ("brace", format!("{}x{}", "{".repeat(n), "}".repeat(n))),
            ("sqrt chain", format!("{} x", r"\sqrt".repeat(n))),
            (
                "superscript",
                format!("{}x{}", "x^{".repeat(n), "}".repeat(n)),
            ),
            (
                "left",
                format!("{}x{}", r"\left(".repeat(n), r"\right)".repeat(n)),
            ),
            (
                "env",
                format!(
                    "{}x{}",
                    r"\begin{matrix}".repeat(n),
                    r"\end{matrix}".repeat(n)
                ),
            ),
            (
                "sqrt optional",
                format!("{}x{}", r"\sqrt[".repeat(n), "]{y}".repeat(n)),
            ),
            ("unbraced frac", format!("{}x", r"\frac1".repeat(n))),
            (
                "frac of left",
                format!("{}x", r"\frac\left(x\right)".repeat(n)),
            ),
            (
                "textcolor",
                format!("{}x{}", r"\textcolor{red}{".repeat(n), "}".repeat(n)),
            ),
            (
                "genfrac",
                format!("{}x", r"\genfrac(){0pt}{0}{1}".repeat(n)),
            ),
            (
                "begingroup",
                format!("{} x{}", r"\begingroup".repeat(n), r"\endgroup".repeat(n)),
            ),
        ]
    }

    #[test]
    fn everyday_formulas_pass() {
        for f in [
            "E=mc^2",
            r"\int_0^1 x^2\,dx = \frac{1}{3}",
            r"\begin{pmatrix} a & b \\ c & d \end{pmatrix}",
            r"\left( \sum_{i=1}^{n} i \right)^2",
            r"\sqrt[3]{\frac{a}{b}}",
            r"f(x) = \begin{cases} 1 & x > 0 \\ 0 & \text{otherwise} \end{cases}",
            r"\mathbb{R} \to \mathcal{C}",
            r"\{ x \mid x > 0 \}",
            r"\frac12 + \frac34 - \sqrt2",
            r"x' + f''(x) = [0, 1) \cup (2, 3]",
            r"\xrightarrow[below]{above} \overset{!}{=} \binom{n}{k}",
            r"\frac{\partial^2 u}{\partial t^2} = c^2 \nabla^2 u",
        ] {
            assert_eq!(check_formula(f), Ok(()), "{f}");
        }
    }

    #[test]
    fn long_flat_formulas_are_not_nesting() {
        for flat in [
            r"\alpha+".repeat(500),
            r"\frac12+".repeat(300),
            r"\alpha\beta\gamma\delta".repeat(100),
            r"a \le b \times c \cdot ".repeat(100),
            r"x^2_1 ".repeat(300),
            r"(a)[b]".repeat(300),
        ] {
            assert_eq!(check_formula(&flat), Ok(()), "{}", &flat[..40]);
        }
    }

    #[test]
    fn depth_is_capped_at_the_limit_not_before() {
        let braces = |n: usize| format!("{}x{}", "{".repeat(n), "}".repeat(n));
        assert_eq!(check_formula(&braces(MAX_MATH_DEPTH)), Ok(()));
        assert_eq!(
            check_formula(&braces(MAX_MATH_DEPTH + 1)),
            Err(GuardError::TooDeep)
        );
    }

    /// Every pattern, at the security review's depths and well past the
    /// limit, is refused, including the two that bypassed the first guard:
    /// `\sqrt[` (brackets were not counted) and `\frac1\frac1…` (unbraced
    /// arguments nest with no brace at all).
    #[test]
    fn hostile_depths_are_refused() {
        for n in [MAX_MATH_DEPTH + 1, 200, 500] {
            for (name, latex) in patterns(n) {
                if latex.len() > MAX_MATH_BYTES {
                    continue;
                }
                assert_eq!(
                    check_formula(&latex),
                    Err(GuardError::TooDeep),
                    "{name} x{n}"
                );
            }
        }
        let braces = format!("{}x{}", "{".repeat(20_000), "}".repeat(20_000));
        assert_eq!(check_formula(&braces), Err(GuardError::TooLong(40_001)));
    }

    /// The guard's real job: anything it accepts converts on a 1 MiB stack
    /// (wasm's default) without overflowing. For each pattern, find the
    /// deepest size the guard accepts and convert exactly that. An overflow
    /// here aborts the test process, which is the point.
    #[test]
    fn the_deepest_accepted_input_converts_on_a_wasm_sized_stack() {
        for (name, _) in patterns(1) {
            let deepest = (1..=MAX_MATH_DEPTH + 1)
                .take_while(|n| {
                    let (_, latex) = patterns(*n).into_iter().find(|(p, _)| *p == name).unwrap();
                    check_formula(&latex).is_ok()
                })
                .last()
                .unwrap_or_else(|| panic!("{name}: even one level is refused"));
            let (_, latex) = patterns(deepest)
                .into_iter()
                .find(|(p, _)| *p == name)
                .unwrap();
            let converted = std::thread::Builder::new()
                .stack_size(1 << 20)
                .spawn(move || {
                    math_core::LatexToMathML::new(math_core::MathCoreConfig::default(), vec![])
                        .unwrap()
                        .convert_with_local_state(&latex, math_core::MathDisplay::Block)
                        .is_ok()
                })
                .unwrap()
                .join()
                .unwrap();
            assert!(converted, "{name} x{deepest} should convert");
        }
    }

    #[test]
    fn macro_definitions_are_refused() {
        for (f, name) in [
            (r"\newcommand{\R}{\mathbb{R}} \R", "newcommand"),
            (r"\renewcommand\x{y}", "renewcommand"),
            (r"\def\a{\a\a}\a", "def"),
            (r"\let\b=\a", "let"),
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
        assert_eq!(check_formula(r"\é + \ü"), Ok(()));
    }

    #[test]
    fn the_arity_table_is_sorted_and_knows_the_common_commands() {
        let table = super::super::arity::ARGUMENT_COMMANDS;
        assert!(
            table.windows(2).all(|w| w[0].0 < w[1].0),
            "sorted for binary search"
        );
        for (name, n) in [
            ("frac", 2),
            ("sqrt", 1),
            ("genfrac", 6),
            ("textcolor", 2),
            ("text", 1),
        ] {
            assert_eq!(arguments(name), n, "{name}");
        }
        for name in ["alpha", "le", "cdot", "notacommand"] {
            assert_eq!(arguments(name), 0, "{name}");
        }
    }
}
