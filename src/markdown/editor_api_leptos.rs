//! Leptos-signal-backed implementation of [`EditorApi`].
//!
//! Browser consumers (llm-wiki, custom embedders, AI agents running
//! in the same WASM module as the editor) plug into the editor by
//! holding a [`SignalEditorApi`] instead of poking `RwSignal<String>`
//! directly.  Operations route through the trait, so the same code
//! that mutates the desktop EDIT control via win32 messaging can
//! mutate a browser textarea via signal updates — without knowing
//! which surface it's talking to.
//!
//! The trait's `&self` methods need interior mutability; Leptos
//! signals already provide that, so the impl is mostly a thin
//! adapter.  Undo history is tracked locally (signals don't ship
//! built-in undo).

use std::cell::RefCell;
use std::ops::Range;
use std::rc::Rc;

use editmark_core::editor_api::{
    DocumentChange, DocumentMetadata, EditError, EditorApi, Selection, SubscriptionId,
};
use leptos::prelude::*;

/// EditorApi over a `RwSignal<String>` source plus sibling signals
/// for selection / metadata.  Cheap to clone — all internal state is
/// behind `Rc`.
#[derive(Clone)]
pub struct SignalEditorApi {
    source: RwSignal<String>,
    selection: RwSignal<Selection>,
    metadata: RwSignal<DocumentMetadata>,
    inner: Rc<RefCell<SignalInner>>,
}

/// A registered change listener: its subscription id paired with the callback.
type ListenerEntry = (SubscriptionId, Box<dyn Fn(&DocumentChange)>);

struct SignalInner {
    undo_stack: Vec<UndoEntry>,
    listeners: Vec<ListenerEntry>,
}

struct UndoEntry {
    new_range: Range<usize>,
    original_text: String,
}

impl SignalEditorApi {
    /// Build an EditorApi bound to an existing `source` signal.  The
    /// `selection` and `metadata` signals are created fresh and the
    /// caller can read them via [`Self::selection_signal`] /
    /// [`Self::metadata_signal`] if they want to display selection
    /// or dirty state.
    pub fn new(source: RwSignal<String>) -> Self {
        Self {
            source,
            selection: RwSignal::new(Selection::default()),
            metadata: RwSignal::new(DocumentMetadata::default()),
            inner: Rc::new(RefCell::new(SignalInner {
                undo_stack: Vec::new(),
                listeners: Vec::new(),
            })),
        }
    }

    /// Build with caller-provided selection + metadata signals — use
    /// this when the consumer already holds those signals (e.g.
    /// for displaying a status bar bound to `metadata.dirty`).
    pub fn with_signals(
        source: RwSignal<String>,
        selection: RwSignal<Selection>,
        metadata: RwSignal<DocumentMetadata>,
    ) -> Self {
        Self {
            source,
            selection,
            metadata,
            inner: Rc::new(RefCell::new(SignalInner {
                undo_stack: Vec::new(),
                listeners: Vec::new(),
            })),
        }
    }

    pub fn source_signal(&self) -> RwSignal<String> {
        self.source
    }
    pub fn selection_signal(&self) -> RwSignal<Selection> {
        self.selection
    }
    pub fn metadata_signal(&self) -> RwSignal<DocumentMetadata> {
        self.metadata
    }

    /// Listener count — exposed for tests.
    pub fn listener_count(&self) -> usize {
        self.inner.borrow().listeners.len()
    }

    fn notify(&self, change: DocumentChange) {
        // Pull listeners out, fire, put back — same re-entrancy
        // safety as `InMemoryEditor`.
        let listeners = std::mem::take(&mut self.inner.borrow_mut().listeners);
        for (_, listener) in &listeners {
            listener(&change);
        }
        let mut inner = self.inner.borrow_mut();
        let added = std::mem::take(&mut inner.listeners);
        inner.listeners = listeners;
        inner.listeners.extend(added);
    }
}

impl EditorApi for SignalEditorApi {
    fn read_document(&self) -> String {
        self.source.get_untracked()
    }

    fn document_len(&self) -> usize {
        self.source.with_untracked(|s| s.len())
    }

    fn get_selection(&self) -> Selection {
        self.selection.get_untracked()
    }

    fn set_selection(&self, sel: Selection) {
        self.selection.set(sel);
    }

    fn replace_range(&self, range: Range<usize>, text: &str) -> Result<(), EditError> {
        if range.start > range.end {
            return Err(EditError::InvertedRange {
                start: range.start,
                end: range.end,
            });
        }

        // Validate + apply against the signal — use update so the
        // mutation is observable to subscribers.
        let result = self.source.try_update(|doc| -> Result<_, EditError> {
            if range.end > doc.len() {
                return Err(EditError::OutOfBounds {
                    start: range.start,
                    end: range.end,
                    doc_len: doc.len(),
                });
            }
            if !doc.is_char_boundary(range.start) {
                return Err(EditError::NotOnCharBoundary {
                    offset: range.start,
                });
            }
            if !doc.is_char_boundary(range.end) {
                return Err(EditError::NotOnCharBoundary { offset: range.end });
            }
            let original: String = doc[range.clone()].to_string();
            doc.replace_range(range.clone(), text);
            Ok(original)
        });

        let original = match result {
            Some(Ok(orig)) => orig,
            Some(Err(e)) => return Err(e),
            None => return Err(EditError::Backend("source signal is disposed".to_string())),
        };

        let new_range = range.start..(range.start + text.len());

        // Record undo + bump version.
        {
            let mut inner = self.inner.borrow_mut();
            inner.undo_stack.push(UndoEntry {
                new_range: new_range.clone(),
                original_text: original,
            });
        }
        self.metadata.update(|m| {
            m.dirty = true;
            m.version = m.version.wrapping_add(1);
        });

        // Shift selection — same rules as InMemoryEditor.
        self.selection.update(|sel| {
            let sel_start = sel.range.start;
            if range.end <= sel_start {
                let delta = text.len() as isize - (range.end - range.start) as isize;
                let new_start = (sel_start as isize + delta).max(0) as usize;
                let new_end = (sel.range.end as isize + delta).max(0) as usize;
                *sel = Selection {
                    range: new_start..new_end,
                    primary_caret: (sel.primary_caret as isize + delta).max(0) as usize,
                };
            } else {
                *sel = Selection::caret(new_range.end);
            }
        });

        let version = self.metadata.with_untracked(|m| m.version);
        self.notify(DocumentChange {
            range,
            replacement: text.to_string(),
            version,
        });

        Ok(())
    }

    fn undo(&self) -> Result<(), EditError> {
        let entry = self
            .inner
            .borrow_mut()
            .undo_stack
            .pop()
            .ok_or_else(|| EditError::Backend("nothing to undo".to_string()))?;

        self.source.update(|doc| {
            doc.replace_range(entry.new_range.clone(), &entry.original_text);
        });
        self.metadata
            .update(|m| m.version = m.version.wrapping_add(1));

        let version = self.metadata.with_untracked(|m| m.version);
        let restored_range =
            entry.new_range.start..(entry.new_range.start + entry.original_text.len());
        self.notify(DocumentChange {
            range: restored_range,
            replacement: entry.original_text,
            version,
        });
        Ok(())
    }

    fn subscribe(&self, listener: Box<dyn Fn(&DocumentChange)>) -> SubscriptionId {
        let id = SubscriptionId::new();
        self.inner.borrow_mut().listeners.push((id, listener));
        id
    }

    fn unsubscribe(&self, id: SubscriptionId) {
        self.inner
            .borrow_mut()
            .listeners
            .retain(|(sid, _)| *sid != id);
    }

    fn metadata(&self) -> DocumentMetadata {
        self.metadata.get_untracked()
    }
}

#[cfg(test)]
mod tests {
    //! These tests need a Leptos reactive runtime.  Each test sets
    //! one up via `Owner::new` so signals work outside a component
    //! tree.

    use super::*;
    use leptos::reactive::owner::Owner;
    use std::cell::Cell;

    fn with_runtime<F: FnOnce()>(f: F) {
        let owner = Owner::new();
        owner.with(f);
    }

    #[test]
    fn read_returns_current_source() {
        with_runtime(|| {
            let source = RwSignal::new("hello".to_string());
            let api = SignalEditorApi::new(source);
            assert_eq!(api.read_document(), "hello");
            assert_eq!(api.document_len(), 5);
        });
    }

    #[test]
    fn replace_range_mutates_source_signal() {
        with_runtime(|| {
            let source = RwSignal::new("hello world".to_string());
            let api = SignalEditorApi::new(source);
            api.replace_range(6..11, "rust").unwrap();
            assert_eq!(source.get_untracked(), "hello rust");
            assert_eq!(api.metadata().version, 1);
            assert!(api.metadata().dirty);
        });
    }

    #[test]
    fn undo_round_trips_through_signal() {
        with_runtime(|| {
            let source = RwSignal::new("foo".to_string());
            let api = SignalEditorApi::new(source);
            api.append(" bar").unwrap();
            assert_eq!(source.get_untracked(), "foo bar");
            api.undo().unwrap();
            assert_eq!(source.get_untracked(), "foo");
        });
    }

    #[test]
    fn out_of_bounds_replace_does_not_touch_signal() {
        with_runtime(|| {
            let source = RwSignal::new("abc".to_string());
            let api = SignalEditorApi::new(source);
            let err = api.replace_range(2..99, "x").unwrap_err();
            assert!(matches!(err, EditError::OutOfBounds { .. }));
            assert_eq!(source.get_untracked(), "abc");
        });
    }

    #[test]
    fn change_listener_fires_on_signal_mutation() {
        with_runtime(|| {
            let source = RwSignal::new(String::new());
            let api = SignalEditorApi::new(source);
            let fired = Rc::new(Cell::new(0u32));
            let fired_clone = fired.clone();
            api.subscribe(Box::new(move |_| {
                fired_clone.set(fired_clone.get() + 1);
            }));
            api.append("a").unwrap();
            api.append("b").unwrap();
            assert_eq!(fired.get(), 2);
        });
    }

    #[test]
    fn unsubscribe_stops_callbacks() {
        with_runtime(|| {
            let source = RwSignal::new(String::new());
            let api = SignalEditorApi::new(source);
            let fired = Rc::new(Cell::new(0u32));
            let fired_clone = fired.clone();
            let id = api.subscribe(Box::new(move |_| {
                fired_clone.set(fired_clone.get() + 1);
            }));
            api.append("x").unwrap();
            api.unsubscribe(id);
            api.append("y").unwrap();
            assert_eq!(fired.get(), 1);
        });
    }

    #[test]
    fn selection_shifts_under_pre_selection_edit() {
        with_runtime(|| {
            let source = RwSignal::new("AAAA-BBBB".to_string());
            let api = SignalEditorApi::new(source);
            api.set_selection(Selection::range(5, 9));
            api.replace_range(0..4, "X").unwrap();
            let sel = api.get_selection();
            assert_eq!(sel.range, 2..6);
        });
    }
}
