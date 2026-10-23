"""Generate src/markdown/math/arity.rs from math-core 0.9.0's commands.rs.

Usage: python -X utf8 gen_arity.py <path to math-core-0.9.0/src/commands.rs> <out.rs>
"""
import re
import sys

src = open(sys.argv[1], encoding="utf-8").read()

# Kinds that take no argument (symbols, switches, spacing). Anything not
# listed and not given an explicit arity below defaults to 1, so a kind this
# script has never seen is over-counted, never under-counted.
ZERO = {
    "Relation", "Letter", "BinaryOp", "ForceBinaryOp", "Op", "UprightLetter",
    "Ord", "ForceOp", "ForceRelation", "Open", "Close", "ForceOpen",
    "ForceClose", "PseudoOperatorLimits", "ForcePunctuation", "ForceOrd",
    "Prime", "Space", "NonBreakingSpace", "NewLine", "HLine", "Limits",
    "NoNumber", "Relax", "Style", "TransformSwitch", "InfixGenFrac", "Shove",
    "ForceMathInner", "MathOrTextMode",
    # Grouping and definitions are handled by the guard itself.
    "Left", "Right", "GroupBegin", "GroupEnd", "Global", "Def", "Let",
    "NewCommand",
}
ARITY = {
    "Frac": 2, "Binom": 2, "Overset": 2, "Underset": 2, "Sideset": 2,
    "RaiseBox": 2, "Genfrac": 6, "MathChoice": 4, "Prescript": 3,
}

# Zero-argument kinds that print nothing (switches, spacing). The guard
# never lets one of these complete another command's argument, so if
# math-core skips them while reading arguments the guard over-counts.
SILENT = {
    "Relax", "Limits", "NoNumber", "Style", "TransformSwitch", "Shove",
    "Space", "NonBreakingSpace", "NewLine", "HLine",
}

entries = {}
silent = set()
pat = re.compile(r'^\s*((?:"[^"]*"\s*\|\s*)*"[^"]*")\s*=>\s*([A-Za-z]+)(.*)$')
# Join a block arm (`"name" => {` + body on the next line) onto one line.
src = re.sub(r'=>\s*\{\s*\n\s*', '=> ', src)
for line in src.splitlines():
    m = pat.match(line)
    if not m:
        continue
    names = re.findall(r'"([^"]*)"', m.group(1))
    kind, rest = m.group(2), m.group(3)
    if kind == "CustomCmd":
        n = int(re.match(r"\((\d+)", rest).group(1))
    elif kind == "TextMode":
        n = 1 if rest.startswith("(TextToken::Accent") else 0
    elif kind == "Token":
        n = 1 if rest.startswith("::Accent") else 0
    elif kind in ARITY:
        n = ARITY[kind]
    elif kind in ZERO:
        n = 0
    else:
        n = 1
    is_silent = kind in SILENT or (
        kind == "MathOrTextMode" and re.match(r"\(&(Space|NonBreakingSpace)", rest)
    )
    for name in names:
        if not re.fullmatch(r"[A-Za-z]+", name):
            continue
        if n > 0:
            entries[name] = max(n, entries.get(name, 0))
        elif is_silent:
            silent.add(name)

rows = sorted(entries.items())
out = [
    "//! GENERATED from math-core 0.9.0 `src/commands.rs` by",
    "//! `assets/math-core/gen_arity.py` (ldui-r0co); do not hand-edit.",
    "//! Regenerate whenever the `math-core` pin in `Cargo.toml` moves.",
    "//!",
    "//! Every control word math-core 0.9.0 accepts that takes arguments, with",
    "//! the most arguments it can take. Words not listed take none (symbols,",
    "//! switches) or are unknown to math-core, which rejects them at once.",
    "",
    "/// `(name, arguments)`, sorted by name for binary search.",
    "pub(super) const ARGUMENT_COMMANDS: &[(&str, u8)] = &[",
]
out += [f'    ("{n}", {a}),' for n, a in rows]
out += [
    "];",
    "",
    r"/// Zero-argument words that print nothing (`\relax`, `\displaystyle`,",
    r"/// `\quad`, ...), sorted. They never complete another command's argument.",
    "pub(super) const SILENT_COMMANDS: &[&str] = &[",
]
out += [f'    "{n}",' for n in sorted(silent - set(entries))]
out += ["];", ""]
open(sys.argv[2], "w", encoding="utf-8", newline="\n").write("\n".join(out))
print(len(rows), "commands;", sum(1 for _, a in rows if a > 1), "with 2+ arguments;", len(silent - set(entries)), "silent")
