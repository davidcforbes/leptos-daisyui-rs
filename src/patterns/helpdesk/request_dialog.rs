//! The "file a bug or request" dialog: kind, summary, description and
//! screenshots, gated by the host's configuration.

use super::model::*;
use super::state::{NewTicketCaps, NewTicketError, validate_new_ticket};
use super::texts::HelpdeskTexts;
use crate::components::{
    Button, ButtonColor, ButtonStyle, ButtonType, Field, ImageAttachmentField, Input, Modal,
    ModalAction, ModalBox, Textarea,
};
use crate::patterns::{PageStatePanel, PageStatePanelKind, PageStatePanelTexts};
use leptos::html::Input as HtmlInput;
use leptos::prelude::*;
use leptos::web_sys;

/// The dialog's in-progress fields, before attachments are joined in.
#[derive(Clone, Debug, PartialEq, Eq)]
pub(crate) struct Draft {
    /// Bug or Request.
    pub kind: TicketKind,
    /// The summary field's current value.
    pub summary: String,
    /// The description field's current value.
    pub description: String,
}

impl Default for Draft {
    fn default() -> Self {
        Self {
            kind: TicketKind::Bug,
            summary: String::new(),
            description: String::new(),
        }
    }
}

impl Draft {
    /// The summary-only validation message, or `None` if it is acceptable.
    /// Probed with empty description and no images so only the summary rule
    /// can fire.
    pub(crate) fn summary_error(&self, t: &HelpdeskTexts, caps: &NewTicketCaps) -> Option<String> {
        let probe = NewTicket {
            kind: self.kind.clone(),
            summary: self.summary.clone(),
            description: String::new(),
            context: RequestContext::default(),
            images: vec![],
        };
        match validate_new_ticket(&probe, caps) {
            Ok(()) => None,
            Err(errs) => errs.iter().find_map(|e| match e {
                NewTicketError::SummaryBlank => Some(t.summary_required.clone()),
                NewTicketError::SummaryTooLong { max } => {
                    Some(t.summary_too_long.replace("{max}", &max.to_string()))
                }
                _ => None,
            }),
        }
    }

    /// Build the submission from the draft, the request context and the
    /// admitted images.
    pub(crate) fn to_ticket(
        &self,
        ctx: &RequestContext,
        images: Vec<ImageAttachment>,
    ) -> NewTicket {
        NewTicket {
            kind: self.kind.clone(),
            summary: self.summary.trim().to_owned(),
            description: self.description.trim().to_owned(),
            context: ctx.clone(),
            images,
        }
    }
}

/// The New Request dialog. `on_submit` receives the built ticket and a
/// callback the host must call exactly once with the result; the dialog
/// closes on success and shows the error otherwise.
#[component]
pub fn NewRequestDialog(
    /// Whether the dialog is open. Also settable by a host launcher (F2).
    #[prop(into)]
    open: RwSignal<bool>,
    /// The host's current metadata, or `None` before the first load.
    #[prop(into)]
    meta: Signal<Option<HelpdeskMeta>>,
    /// Where the request is being filed from.
    #[prop(into)]
    context: Signal<RequestContext>,
    /// All rendered copy.
    #[prop(optional, into, default = Signal::stored(HelpdeskTexts::default()))]
    texts: Signal<HelpdeskTexts>,
    /// Whether the dialog offers image attachments. Defaults to `true`, so
    /// every existing call site is unchanged. A host whose backend takes no
    /// images sets `false` and the person is never offered a control that
    /// cannot work (ldui-8tlg); the submitted draft then carries no images.
    #[prop(optional, default = true)]
    attachments: bool,
    /// Submits the draft; the second argument resolves with the outcome.
    on_submit: Callback<(NewTicket, Callback<Result<HelpdeskTicket, HelpdeskError>>)>,
) -> impl IntoView {
    let caps = NewTicketCaps::default();
    let draft = RwSignal::new(Draft::default());
    let images: RwSignal<Vec<ImageAttachment>> = RwSignal::new(Vec::new());
    let summary_ref = NodeRef::<HtmlInput>::new();

    // Opening the dialog focuses the summary field (ldui-efuf). Deferred to
    // the next animation frame, like `SearchPickerDialog`, so it runs after
    // Modal's own effect calls `show_modal()`: calling `.focus()` on a
    // descendant before the dialog is the document's top layer is a silent
    // no-op. Closing returns focus to the launcher through the native
    // `<dialog>` close, which Modal drives.
    Effect::new(move |_| {
        if open.get() {
            request_animation_frame(move || {
                if let Some(element) = summary_ref.get_untracked() {
                    let _ = element.focus();
                }
            });
        }
    });
    let touched = RwSignal::new(false);
    let pending = RwSignal::new(false);
    let error = RwSignal::new(Option::<String>::None);

    let configured = Signal::derive(move || meta.get().is_some_and(|m| m.configured));
    let reason = Signal::derive(move || meta.get().and_then(|m| m.unavailable_reason));
    let kinds = Signal::derive(move || {
        meta.get()
            .map(|m| m.kinds)
            .unwrap_or_else(|| vec![TicketKind::Bug, TicketKind::Request])
    });
    let summary_error = Signal::derive(move || {
        if touched.get() {
            draft.get().summary_error(&texts.get(), &caps)
        } else {
            None
        }
    });
    let can_submit = Signal::derive(move || {
        configured.get()
            && !pending.get()
            && draft.get().summary_error(&texts.get(), &caps).is_none()
    });

    let reset = move || {
        draft.set(Draft::default());
        images.set(Vec::new());
        touched.set(false);
        error.set(None);
        pending.set(false);
    };
    let close = move || {
        open.set(false);
        reset();
    };

    let submit = move || {
        touched.set(true);
        if !can_submit.get_untracked() {
            return;
        }
        pending.set(true);
        error.set(None);
        let ticket = draft
            .get_untracked()
            .to_ticket(&context.get_untracked(), images.get_untracked());
        let done = Callback::new(move |r: Result<HelpdeskTicket, HelpdeskError>| match r {
            Ok(_) => close(),
            Err(e) => {
                pending.set(false);
                let t = texts.get_untracked();
                error.set(Some(match e.kind {
                    HelpdeskErrorKind::RateLimited { retry_after_s } => t
                        .rate_limited
                        .replace("{seconds}", &retry_after_s.to_string()),
                    _ => format!("{}: {}", t.submit_failed, e.message),
                }));
            }
        });
        on_submit.run((ticket, done));
    };

    let panel_texts = Signal::derive(move || PageStatePanelTexts {
        forbidden: texts.get().not_configured_title,
        ..PageStatePanelTexts::default()
    });
    let context_line = move || {
        let c = context.get();
        texts
            .get()
            .context_line
            .replace("{route}", &c.route)
            .replace("{build}", &c.build)
    };

    view! {
        <Modal
            open=Signal::derive(move || open.get())
            backdrop=true
            label=Signal::derive(move || Some(texts.get().dialog_title))
            on_close_request=Callback::new(move |_| close())
            attr:data-helpdesk-dialog=""
        >
            <ModalBox class="max-w-2xl">
                <h2 class="ld-text-title mb-4">{move || texts.get().dialog_title}</h2>
                <Show when=move || !configured.get()>
                    <div data-helpdesk-not-configured="">
                        <PageStatePanel kind=PageStatePanelKind::Forbidden texts=panel_texts detail=reason />
                    </div>
                </Show>
                <Show when=move || configured.get()>
                    <form
                        class="flex flex-col gap-4"
                        on:submit=move |ev| {
                            ev.prevent_default();
                            submit();
                        }
                    >
                        <fieldset class="flex flex-col gap-2" data-helpdesk-kind="">
                            <legend class="text-sm font-medium">{move || texts.get().kind}</legend>
                            <div class="join">
                                <For each=move || kinds.get() key=|k| format!("{k:?}") let:k>
                                    {
                                        let k_label = k.clone();
                                        let k_checked = k.clone();
                                        let k_click = k.clone();
                                        let checked = move || draft.get().kind == k_checked;
                                        let on_change = move |_| draft.update(|d| d.kind = k_click.clone());
                                        view! {
                                            <input
                                                type="radio"
                                                name="helpdesk-kind"
                                                class="join-item btn btn-sm"
                                                aria-label=move || texts.get().kind_name(&k_label)
                                                prop:checked=checked
                                                on:change=on_change
                                            />
                                        }
                                    }
                                </For>
                            </div>
                        </fieldset>
                        <Field
                            label=Signal::derive(move || Some(texts.get().summary))
                            error=summary_error
                            required=true
                        >
                            <Input
                                node_ref=summary_ref
                                value=Signal::derive(move || draft.get().summary)
                                maxlength=Some(caps.summary_max as u32)
                                on_input=Callback::new(move |v: String| {
                                    touched.set(true);
                                    draft.update(|d| d.summary = v);
                                })
                                attr:data-helpdesk-summary=""
                            />
                        </Field>
                        <Field label=Signal::derive(move || Some(texts.get().description))>
                            <Textarea
                                value=Signal::derive(move || draft.get().description)
                                rows=Some(5)
                                maxlength=Some(caps.description_max as u32)
                                placeholder=Signal::derive(move || {
                                    let t = texts.get();
                                    if draft.get().kind == TicketKind::Bug {
                                        t.bug_placeholder
                                    } else {
                                        t.description_placeholder
                                    }
                                })
                                on_input=Callback::new(move |v: String| draft.update(|d| d.description = v))
                                attr:data-helpdesk-description=""
                            />
                        </Field>
                        {attachments
                            .then(move || {
                                view! {
                                    <div data-helpdesk-images="">
                                        <ImageAttachmentField
                                            images=images
                                            capture_document_paste=true
                                            disabled=pending
                                        />
                                    </div>
                                }
                            })}
                        <p class="text-sm text-base-content/75" data-helpdesk-context="">
                            {context_line}
                        </p>
                        <p class="text-sm text-error" role="alert" data-helpdesk-submit-error="">
                            {move || error.get().unwrap_or_default()}
                        </p>
                        <ModalAction>
                            <Button
                                style=ButtonStyle::Ghost
                                on_click=Callback::new(move |_: web_sys::MouseEvent| close())
                                attr:data-helpdesk-cancel=""
                            >
                                {move || texts.get().cancel}
                            </Button>
                            <Button
                                color=ButtonColor::Primary
                                button_type=ButtonType::Submit
                                disabled=Signal::derive(move || !can_submit.get())
                                loading=pending
                                attr:data-helpdesk-submit=""
                            >
                                {move || texts.get().submit}
                            </Button>
                        </ModalAction>
                    </form>
                </Show>
            </ModalBox>
        </Modal>
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn draft_builds_a_ticket_and_reports_summary_errors() {
        let mut d = Draft {
            kind: TicketKind::Bug,
            summary: "  ".into(),
            ..Draft::default()
        };
        let ctx = RequestContext {
            route: "/r".into(),
            build: "b".into(),
        };
        let t = HelpdeskTexts::default();
        assert_eq!(
            d.summary_error(&t, &NewTicketCaps::default()),
            Some(t.summary_required.clone())
        );
        d.summary = "Broken".into();
        assert_eq!(d.summary_error(&t, &NewTicketCaps::default()), None);
        let nt = d.to_ticket(&ctx, vec![]);
        assert_eq!(
            (nt.kind, nt.summary.as_str(), nt.context.route.as_str()),
            (TicketKind::Bug, "Broken", "/r")
        );
    }

    #[test]
    fn long_summary_reports_the_length_error() {
        let d = Draft {
            summary: "x".repeat(NewTicketCaps::default().summary_max + 1),
            ..Draft::default()
        };
        let t = HelpdeskTexts::default();
        let err = d.summary_error(&t, &NewTicketCaps::default()).unwrap();
        assert!(err.contains("200"), "{err}");
    }
}
