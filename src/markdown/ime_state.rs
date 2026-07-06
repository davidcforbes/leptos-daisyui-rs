//! IME composition state machine for the graphic-mode editor (em-berj.2).
//!
//! While an IME composition is active — Japanese, Chinese, Korean, or
//! the long-press / dictation popups on macOS / iOS — the browser
//! mutates the DOM continuously with preedit text and `input` events
//! fire for *every keystroke* of the composition.  Dispatching those
//! through the edit funnel would:
//!
//! 1. Corrupt the canonical markdown source (each preedit step is a
//!    half-formed glyph; the IME may also rewrite earlier characters).
//! 2. Force the funnel to re-render the DOM, which the IME interprets
//!    as the user committing — *every* keystroke ends the composition.
//!
//! Both well-known IME footguns: see Muya's `composition_start_handler`
//! and ProseMirror's `Composition` plugin for prior art.  We follow the
//! same shape: suspend all input dispatch between `compositionstart`
//! and `compositionend`, then dispatch a *single* edit at the end that
//! captures the final committed text from the DOM.
//!
//! This module owns only the shared mutable flag.  `graphic_editor.rs`
//! wires it into the actual DOM event listeners.

use std::cell::Cell;
use std::rc::Rc;

/// Shared composition flag — `true` between `compositionstart` and
/// `compositionend`.  Cloned cheaply into each event closure so they
/// all observe the same state.
#[derive(Clone, Default)]
pub struct ImeState {
    composing: Rc<Cell<bool>>,
}

impl ImeState {
    pub fn new() -> Self {
        Self::default()
    }

    /// `true` while the browser's IME is mid-composition.  The
    /// `on_input` handler MUST bail when this returns `true` — see
    /// the module doc for why.
    pub fn is_composing(&self) -> bool {
        self.composing.get()
    }

    /// Called from the `compositionstart` listener.
    pub fn begin(&self) {
        self.composing.set(true);
    }

    /// Called from the `compositionend` listener.  Returns `true` if
    /// the state actually transitioned (false if `compositionend`
    /// fired without a preceding `compositionstart`, which some
    /// browsers will do for dead-key sequences — see Spanish `ñ` on
    /// Windows).  Callers can use the return value to decide whether
    /// to dispatch the trailing edit.
    pub fn end(&self) -> bool {
        self.composing.replace(false)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn defaults_to_not_composing() {
        let s = ImeState::new();
        assert!(!s.is_composing());
    }

    #[test]
    fn begin_then_end_transitions_flag() {
        let s = ImeState::new();
        s.begin();
        assert!(s.is_composing());
        assert!(s.end());
        assert!(!s.is_composing());
    }

    #[test]
    fn end_without_begin_reports_no_transition() {
        let s = ImeState::new();
        assert!(!s.end());
        assert!(!s.is_composing());
    }

    #[test]
    fn clones_share_state() {
        let a = ImeState::new();
        let b = a.clone();
        a.begin();
        assert!(b.is_composing());
        b.end();
        assert!(!a.is_composing());
    }
}
