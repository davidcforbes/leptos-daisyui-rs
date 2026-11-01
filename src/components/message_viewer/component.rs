use super::types::{
    DeliveryStatus, MessageViewerData, MessageViewerMode, MessageViewerSelection,
    MessageViewerTexts, chrome_badge, counts_line, focus_wrap_target, offers_template,
    offers_text_only, selected_index, viewer_message, viewer_placeholders,
};
use crate::components::{
    Badge, BadgeColor, BadgeSize, Button, ButtonColor, ButtonShape, ButtonSize, ButtonStyle,
    EntityRowActionTexts, Icon, IconSize, Join, MessageDirection, MessageHeader, MockupPhone,
    MockupPhoneCamera, MockupPhoneDisplay, Modal, ModalBox, ModalCloseProposal, ModalTexts,
    RichMessage, RichMessageBubble, RichMessageThread, RowActionButton, RowActionKind, Tooltip,
};
use crate::merge_classes;
use leptos::{html, prelude::*};
use std::sync::atomic::{AtomicUsize, Ordering};
use wasm_bindgen::JsCast;

static NEXT_ID: AtomicUsize = AtomicUsize::new(0);

fn next_id(prefix: &str) -> String {
    format!("{prefix}-{}", NEXT_ID.fetch_add(1, Ordering::Relaxed))
}

/// What can take keyboard focus, before the visibility filter.
const TABBABLE: &str = "a[href], button:not([disabled]), input:not([disabled]):not([type='hidden']), select:not([disabled]), textarea:not([disabled]), [tabindex]:not([tabindex='-1'])";

/// Rendered and not `visibility: hidden` (a closed daisyUI `.modal` keeps a
/// real box but is hidden, so a rect alone would count its buttons).
fn is_shown(el: &web_sys::HtmlElement) -> bool {
    let rect = el.get_bounding_client_rect();
    if rect.width() <= 0.0 && rect.height() <= 0.0 {
        return false;
    }
    web_sys::window()
        .and_then(|w| w.get_computed_style(el).ok().flatten())
        .is_none_or(|style| {
            style.get_property_value("visibility").ok().as_deref() != Some("hidden")
                && style.get_property_value("display").ok().as_deref() != Some("none")
        })
}

/// Keeps Tab and Shift+Tab inside the innermost open dialog. A native modal
/// `<dialog>` makes the page inert but still lets Tab leave for the browser
/// chrome, which is not a focus trap. A Tab another listener already handled
/// (the nested picture dialog's) is left alone.
fn trap_tab(event: web_sys::KeyboardEvent) {
    if event.key() != "Tab" || event.default_prevented() {
        return;
    }
    let Some(target) = event
        .target()
        .and_then(|t| t.dyn_into::<web_sys::Element>().ok())
    else {
        return;
    };
    let Ok(Some(dialog)) = target.closest("dialog[open]") else {
        return;
    };
    let Ok(list) = dialog.query_selector_all(TABBABLE) else {
        return;
    };
    let items: Vec<web_sys::HtmlElement> = (0..list.length())
        .filter_map(|i| list.item(i))
        .filter_map(|node| node.dyn_into::<web_sys::HtmlElement>().ok())
        .filter(|el| {
            el.closest("dialog[open]").ok().flatten().as_ref() == Some(&dialog) && is_shown(el)
        })
        .collect();
    let active = web_sys::window()
        .and_then(|w| w.document())
        .and_then(|d| d.active_element());
    let current = active.and_then(|a| {
        items.iter().position(|el| {
            let el: &web_sys::Element = el.as_ref();
            *el == a
        })
    });
    if let Some(next) = focus_wrap_target(items.len(), current, event.shift_key()) {
        event.prevent_default();
        let _ = items[next].focus();
    }
}

/// The switches a message offers. Only this shape re-renders the switch
/// row, so pressing a switch never rebuilds the button that has focus.
#[derive(Clone, Debug, PartialEq, Eq)]
struct SwitchShape {
    languages: Vec<String>,
    text_only: bool,
    template: bool,
}

/// One option of a switch group: a toggle button whose `aria-pressed` follows
/// the selection.
fn switch_button(
    label: String,
    pressed: Signal<bool>,
    value: &'static str,
    on_press: Callback<()>,
) -> impl IntoView {
    view! {
        <Button
            size=ButtonSize::Sm
            color=Signal::derive(move || {
                if pressed.get() { ButtonColor::Neutral } else { ButtonColor::Default }
            })
            style=Signal::derive(move || {
                if pressed.get() { ButtonStyle::Default } else { ButtonStyle::Outline }
            })
            class="join-item"
            on_click=Callback::new(move |_| on_press.run(()))
            attr:aria-pressed=move || if pressed.get() { "true" } else { "false" }
            attr:data-viewer-option=value
        >
            {label}
        </Button>
    }
}

fn switch_group(kind: &'static str, label: String, options: Vec<AnyView>) -> impl IntoView {
    view! {
        <div role="group" aria-label=label data-viewer-switch=kind>
            <Join>{options}</Join>
        </div>
    }
}

fn status_chip(
    status: DeliveryStatus,
    reason: Option<String>,
    texts: &MessageViewerTexts,
) -> AnyView {
    let label = status.label(texts).to_string();
    match reason.filter(|r| !r.trim().is_empty()) {
        Some(reason) => {
            let id = next_id("ldui-message-status-reason");
            let described_by = id.clone();
            view! {
                <Tooltip tip=reason.clone()>
                    <Badge
                        size=BadgeSize::Sm
                        color=status.color()
                        class="ld-focus-ring"
                        attr:tabindex="0"
                        attr:aria-describedby=described_by
                        attr:data-delivery-status=status.as_str()
                    >
                        {label}
                    </Badge>
                </Tooltip>
                <span id=id class="sr-only select-none" data-delivery-status-reason="true">
                    {reason}
                </span>
            }
            .into_any()
        }
        None => view! {
            <Badge
                size=BadgeSize::Sm
                color=status.color()
                attr:data-delivery-status=status.as_str()
            >
                {label}
            </Badge>
        }
        .into_any(),
    }
}

/// # Message Viewer Component
///
/// A read-only view of a SENT message as the recipient saw it, for history
/// screens (ldui-mhur). It composes [`RichMessageBubble`] (the one bubble
/// renderer) inside daisyUI's phone mockup, with channel chrome on top:
///
/// - **WhatsApp:** the business name and number, and a "Template" badge for
///   an approved template or "Session message" for a session message.
/// - **SMS / MMS:** the sending number, an "MMS" badge when the message
///   carries a picture, and the bubble's segment indicator.
///
/// The picture sits above the text with a loading skeleton, a fallback tile
/// that says it could not be shown, and click-to-enlarge (a nested dialog).
/// A picture without alt text gets "{channel} picture" rather than none.
///
/// **Switches** appear only for what the host supplied: a language switch
/// for two or more [`MessageVersion`](super::MessageVersion)s, "With picture
/// / Text only" when `text_only_variant` is set and the version has a
/// picture, and "As sent / Template" when the version has a template (whose
/// placeholders are highlighted). Each switch is a group of `aria-pressed`
/// buttons, so the keyboard reaches every option.
///
/// Under the phone, a details list gives the channel, sent time, sender,
/// delivery status (a chip whose reason is its tooltip and accessible
/// description) and campaign reach; the recipient's replies follow the sent
/// message as received bubbles.
///
/// Every string comes from `texts` ([`MessageViewerTexts`], English by
/// default, [`MessageViewerTexts::es`] for Spanish). The component fetches
/// nothing but the picture URLs the host passes, and reads no clock.
///
/// ```rust,ignore
/// let data = MessageViewerData {
///     channel: MessageChannel::Whatsapp,
///     sender_name: "Diener Law".into(),
///     whatsapp_template: true,
///     versions: vec![MessageVersion { label: "EN".into(), sent, template: Some(template) }],
///     status: Some(DeliveryStatus::Delivered),
///     ..Default::default()
/// };
/// view! { <MessageViewer data=data /> }
/// ```
///
/// ### Add to `input.css`
/// ```css
/// @source inline("mockup-phone mockup-phone-camera mockup-phone-display join join-item btn btn-sm btn-outline btn-neutral badge badge-sm skeleton modal modal-box");
/// ```
///
/// ## Node References
/// - `node_ref` - References the root `<div>` element ([HTMLDivElement](https://developer.mozilla.org/en-US/docs/Web/API/HTMLDivElement))
#[component]
pub fn MessageViewer(
    /// The message to show.
    #[prop(into)]
    data: Signal<MessageViewerData>,

    /// Every rendered string. Defaults to English.
    #[prop(into, default = Signal::stored(MessageViewerTexts::default()))]
    texts: Signal<MessageViewerTexts>,

    /// Draw the phone frame. `false` shows the same screen in a plain
    /// bordered box.
    #[prop(into, default = Signal::stored(true))]
    frame: Signal<bool>,

    /// Additional CSS classes for the root `<div>`.
    #[prop(optional, into)]
    class: &'static str,

    /// Node reference for the root `<div>`.
    #[prop(optional)]
    node_ref: NodeRef<html::Div>,
) -> impl IntoView {
    let selection = RwSignal::new(MessageViewerSelection::initial());
    // Another message (the dialog reopened on a different row) starts over
    // on its first version, with its picture, as sent.
    Effect::watch(
        move || data.track(),
        move |_, _, _| selection.set(MessageViewerSelection::initial()),
        false,
    );

    let message: Memo<Option<RichMessage>> =
        Memo::new(move |_| data.with(|d| texts.with(|t| viewer_message(d, selection.get(), t))));
    let bubble_message = Signal::derive(move || message.get().unwrap_or_default());
    let bubble_texts = Signal::derive(move || texts.with(MessageViewerTexts::bubble_texts));
    let placeholders =
        Signal::derive(move || data.with(|d| viewer_placeholders(d, selection.get())));
    let enlarge_label = Signal::derive(move || texts.with(|t| t.enlarge_picture.clone()));

    // The enlarged picture: `(src, alt)` while open.
    let lightbox = RwSignal::new(None::<(String, String)>);
    let on_image_open = Callback::new(move |header: MessageHeader| {
        if let MessageHeader::Image { src, alt, .. } = header {
            lightbox.set(Some((src, alt)));
        }
    });

    let shown_channel = move || message.with(|m| m.as_ref().map_or("", |m| m.channel.as_str()));

    let chrome = move || {
        let t = texts.get();
        data.with(|d| {
            let badge = message.with(|m| m.as_ref().and_then(|m| chrome_badge(d, m, &t)));
            let number = d.sender_number.clone().map(|n| {
                view! {
                    <span class="truncate text-xs text-base-content/75" data-viewer-number="true">
                        {n}
                    </span>
                }
            });
            view! {
                <header
                    class=move || {
                        if frame.get() {
                            "flex items-center gap-3 border-b border-base-300 bg-base-200 px-4 pt-12 pb-3"
                        } else {
                            "flex items-center gap-3 border-b border-base-300 bg-base-200 px-4 py-3"
                        }
                    }
                    data-viewer-chrome=d.channel.as_str()
                >
                    <div class="flex min-w-0 flex-1 flex-col">
                        <span class="truncate font-semibold" data-viewer-sender="true">
                            {d.sender_name.clone()}
                        </span>
                        {number}
                    </div>
                    {badge.map(|b| {
                        view! {
                            <Badge size=BadgeSize::Sm color=BadgeColor::Neutral attr:data-viewer-badge="true">
                                {b}
                            </Badge>
                        }
                    })}
                </header>
            }
        })
    };

    let replies = move || {
        data.with(|d| {
            (!d.replies.is_empty()).then(|| {
                let messages: Vec<RichMessage> = d
                    .replies
                    .iter()
                    .cloned()
                    .map(|mut reply| {
                        reply.direction = MessageDirection::In;
                        reply
                    })
                    .collect();
                view! {
                    <RichMessageThread
                        messages=messages
                        label=Signal::derive(move || texts.with(|t| t.replies.clone()))
                        texts=bubble_texts
                        attr:data-viewer-replies="true"
                    />
                }
            })
        })
    };

    let screen = move || {
        view! {
            <div class="flex h-full min-h-0 w-full flex-col bg-base-100 text-base-content">
                {chrome}
                <div
                    class="ld-focus-ring flex min-h-0 flex-1 flex-col gap-4 overflow-y-auto p-3"
                    role="region"
                    tabindex="0"
                    aria-label=move || texts.with(|t| t.phone_label.clone())
                    data-viewer-screen="true"
                >
                    <Show
                        when=move || message.with(Option::is_some)
                        fallback=move || {
                            view! {
                                <p class="text-sm text-base-content/75" data-viewer-empty="true">
                                    {move || texts.with(|t| t.no_message.clone())}
                                </p>
                            }
                        }
                    >
                        <RichMessageBubble
                            message=bubble_message
                            texts=bubble_texts
                            placeholders=placeholders
                            show_sms_indicator=Signal::stored(true)
                            image_placeholder=true
                            on_image_open=on_image_open
                            image_open_label=enlarge_label
                        />
                    </Show>
                    {replies}
                </div>
            </div>
        }
    };

    let device = move || {
        if frame.get() {
            view! {
                <MockupPhone class="w-80 max-w-full">
                    <MockupPhoneCamera>""</MockupPhoneCamera>
                    <MockupPhoneDisplay>{screen()}</MockupPhoneDisplay>
                </MockupPhone>
            }
            .into_any()
        } else {
            view! {
                <div class="flex w-80 max-w-full overflow-hidden rounded-box border border-base-300">
                    {screen()}
                </div>
            }
            .into_any()
        }
    };

    let switch_shape = Memo::new(move |_| {
        data.with(|d| {
            let s = selection.get();
            SwitchShape {
                languages: if d.versions.len() > 1 {
                    d.versions.iter().map(|v| v.label.clone()).collect()
                } else {
                    Vec::new()
                },
                text_only: offers_text_only(d, s),
                template: offers_template(d, s),
            }
        })
    });

    let switches = move || {
        let shape = switch_shape.get();
        let t = texts.get();
        let shown_version = move || data.with(|d| selected_index(d, selection.get()));
        let language = (!shape.languages.is_empty()).then(|| {
            let options = shape
                .languages
                .iter()
                .cloned()
                .enumerate()
                .map(|(index, label)| {
                    switch_button(
                        label,
                        Signal::derive(move || shown_version() == index),
                        "language",
                        Callback::new(move |()| selection.update(|s| s.version = index)),
                    )
                    .into_any()
                })
                .collect();
            switch_group("language", t.language.clone(), options)
        });
        let picture = shape.text_only.then(|| {
            let options = [(true, t.with_picture.clone()), (false, t.text_only.clone())]
                .into_iter()
                .map(|(with_picture, label)| {
                    switch_button(
                        label,
                        Signal::derive(move || selection.get().with_picture == with_picture),
                        if with_picture {
                            "with-picture"
                        } else {
                            "text-only"
                        },
                        Callback::new(move |()| {
                            selection.update(|s| s.with_picture = with_picture)
                        }),
                    )
                    .into_any()
                })
                .collect();
            switch_group("picture", t.picture.clone(), options)
        });
        let content = shape.template.then(|| {
            let options = [
                (MessageViewerMode::AsSent, t.as_sent.clone()),
                (MessageViewerMode::Template, t.template.clone()),
            ]
            .into_iter()
            .map(|(mode, label)| {
                switch_button(
                    label,
                    Signal::derive(move || selection.get().mode == mode),
                    mode.as_str(),
                    Callback::new(move |()| selection.update(|s| s.mode = mode)),
                )
                .into_any()
            })
            .collect();
            switch_group("content", t.content.clone(), options)
        });
        let any = language.is_some() || picture.is_some() || content.is_some();
        any.then(|| {
            view! {
                <div
                    role="group"
                    aria-label=t.versions.clone()
                    class="flex flex-wrap justify-center gap-3"
                    data-viewer-switches="true"
                >
                    {language}
                    {picture}
                    {content}
                </div>
            }
        })
    };

    let details = move || {
        let t = texts.get();
        data.with(|d| {
            let channel = message.with(|m| {
                t.channel_name(m.as_ref().map_or(d.channel, |m| m.channel))
                    .to_string()
            });
            let sent_at = d.sent_at.clone().map(|at| {
                view! {
                    <dt class="font-semibold">{t.sent_at_term.clone()}</dt>
                    <dd data-viewer-sent-at="true">{at}</dd>
                }
            });
            let status = d.status.map(|status| {
                view! {
                    <dt class="font-semibold">{t.status_term.clone()}</dt>
                    <dd>{status_chip(status, d.status_reason.clone(), &t)}</dd>
                }
            });
            let counts = d.counts.map(|counts| {
                view! {
                    <dt class="font-semibold">{t.recipients_term.clone()}</dt>
                    <dd data-viewer-counts="true">{counts_line(counts, &t)}</dd>
                }
            });
            view! {
                <dl
                    class="grid w-80 max-w-full grid-cols-[auto_minmax(0,1fr)] items-center gap-x-4 gap-y-2 text-sm"
                    data-viewer-details="true"
                >
                    <dt class="font-semibold">{t.channel_term.clone()}</dt>
                    <dd data-viewer-channel-name="true">{channel}</dd>
                    {sent_at}
                    <dt class="font-semibold">{t.from_term.clone()}</dt>
                    <dd class="break-words">{d.sender_name.clone()}</dd>
                    {status}
                    {counts}
                </dl>
            }
        })
    };

    let lightbox_open = Signal::derive(move || lightbox.with(Option::is_some));
    let lightbox_label = Signal::derive(move || {
        lightbox.with(|l| match l {
            Some((_, alt)) if !alt.trim().is_empty() => alt.clone(),
            _ => texts.with(|t| t.enlarge_picture.clone()),
        })
    });
    let close_lightbox = move || lightbox.set(None);

    view! {
        <div
            node_ref=node_ref
            class=merge_classes!("flex flex-col items-center gap-4", class)
            data-message-viewer="true"
            data-viewer-channel=shown_channel
            data-viewer-mode=move || selection.get().mode.as_str()
        >
            {device}
            {switches}
            {details}
            // No backdrop: this dialog can sit inside MessageViewerDialog's,
            // and a backdrop's `submit` would bubble up and close that one too.
            <Modal
                open=lightbox_open
                label=lightbox_label
                on_close_request=Callback::new(move |_: ModalCloseProposal| close_lightbox())
                on:keydown=trap_tab
                attr:data-viewer-lightbox="true"
            >
                <ModalBox class="flex max-w-3xl flex-col gap-3">
                    <div class="flex justify-end">
                        <Button
                            size=ButtonSize::Sm
                            style=ButtonStyle::Ghost
                            shape=ButtonShape::Square
                            on_click=Callback::new(move |_| close_lightbox())
                            attr:aria-label=move || texts.with(|t| t.close_picture.clone())
                            attr:data-viewer-lightbox-close="true"
                        >
                            <Icon name="x" size=IconSize::Small />
                        </Button>
                    </div>
                    {move || {
                        lightbox
                            .get()
                            .map(|(src, alt)| {
                                view! {
                                    <img
                                        class="mx-auto block h-auto max-h-[75vh] max-w-full rounded-box"
                                        src=src
                                        alt=alt
                                        data-viewer-lightbox-image="true"
                                    />
                                }
                            })
                    }}
                </ModalBox>
            </Modal>
        </div>
    }
}

/// # Message Viewer Dialog Component
///
/// [`MessageViewer`] inside the house [`Modal`], sized for the phone frame:
/// a visible title that names the dialog, a close button, Escape and the
/// backdrop all propose a close through `on_close` (controlled, so the
/// caller's `open` is the only thing that shuts it and focus returns to the
/// trigger). Tab and Shift+Tab wrap inside the dialog: a native modal
/// `<dialog>` alone lets Tab leave for the browser chrome.
///
/// Open it from a table row with [`ViewMessageRowAction`]:
///
/// ```rust,ignore
/// let viewing = RwSignal::new(None::<MessageViewerData>);
/// let column = EntityColumn::action("view", "", |_: &Campaign| String::new()).render_with(
///     move |row: &Campaign| {
///         let data = row.viewer_data();
///         view! {
///             <EntityRowAction action_id="view-message">
///                 <ViewMessageRowAction
///                     name=row.name.clone()
///                     on_click=move |()| viewing.set(Some(data.clone()))
///                 />
///             </EntityRowAction>
///         }
///         .into_any()
///     },
/// );
/// view! {
///     <MessageViewerDialog
///         open=Signal::derive(move || viewing.with(Option::is_some))
///         on_close=move |()| viewing.set(None)
///         title="Spring reminder"
///         data=Signal::derive(move || viewing.get().unwrap_or_default())
///     />
/// }
/// ```
///
/// ### Add to `input.css`
/// See [`MessageViewer`].
///
/// ## Node References
/// - `node_ref` - References the `<dialog>` element ([HTMLDialogElement](https://developer.mozilla.org/en-US/docs/Web/API/HTMLDialogElement))
#[component]
pub fn MessageViewerDialog(
    /// Whether the dialog is open.
    #[prop(into)]
    open: Signal<bool>,

    /// A close was requested (close button, Escape or backdrop). Set `open`
    /// to `false` to accept it.
    #[prop(into)]
    on_close: Callback<()>,

    /// The dialog's visible title, which is also its accessible name.
    #[prop(into)]
    title: Signal<String>,

    /// The message to show.
    #[prop(into)]
    data: Signal<MessageViewerData>,

    /// Every rendered string. Defaults to English.
    #[prop(into, default = Signal::stored(MessageViewerTexts::default()))]
    texts: Signal<MessageViewerTexts>,

    /// Additional CSS classes for the `<dialog>`.
    #[prop(optional, into)]
    class: &'static str,

    /// Node reference for the `<dialog>`.
    #[prop(optional)]
    node_ref: NodeRef<html::Dialog>,
) -> impl IntoView {
    let title_id = next_id("ldui-message-viewer-title");
    let modal_texts = Signal::derive(move || {
        texts.with(|t| ModalTexts {
            backdrop_close: t.backdrop_close.clone(),
        })
    });
    view! {
        <Modal
            open=open
            backdrop=true
            texts=modal_texts
            labelled_by=title_id.clone()
            on_close_request=Callback::new(move |_: ModalCloseProposal| on_close.run(()))
            class=class
            node_ref=node_ref
            on:keydown=trap_tab
            attr:data-message-viewer-dialog="true"
        >
            <ModalBox class="flex w-auto max-w-md flex-col gap-4">
                <div class="flex items-start justify-between gap-4">
                    <h2 id=title_id class="ld-text-title font-semibold" data-viewer-title="true">
                        {move || title.get()}
                    </h2>
                    <Button
                        size=ButtonSize::Sm
                        style=ButtonStyle::Ghost
                        shape=ButtonShape::Square
                        on_click=Callback::new(move |_| on_close.run(()))
                        attr:aria-label=move || texts.with(|t| t.close.clone())
                        attr:data-viewer-close="true"
                    >
                        <Icon name="x" size=IconSize::Small />
                    </Button>
                </div>
                <MessageViewer data=data texts=texts />
            </ModalBox>
        </Modal>
    }
}

/// The row-action labels [`ViewMessageRowAction`] uses: the English row
/// defaults with `open` set to [`MessageViewerTexts::view_message`].
pub fn view_message_row_texts(texts: &MessageViewerTexts) -> EntityRowActionTexts {
    EntityRowActionTexts {
        open: texts.view_message.clone(),
        ..EntityRowActionTexts::default()
    }
}

/// # View Message Row Action Component
///
/// The EntityTable row action that opens a [`MessageViewerDialog`]: the
/// canonical eye [`RowActionButton`] ([`RowActionKind::Open`]) named "View
/// message: {name}" ("Ver mensaje: {name}" with the Spanish table), with the
/// same text as its tooltip and `data-row-action="open"`. Place it inside
/// an `EntityRowAction` slot (see [`MessageViewerDialog`]).
#[component]
pub fn ViewMessageRowAction(
    /// The row's display name, substituted for `{name}`.
    #[prop(into)]
    name: Signal<String>,

    /// Opens the viewer; the row identity is the caller's to capture.
    #[prop(into)]
    on_click: Callback<()>,

    /// Why the message cannot be viewed for this row; blank keeps it enabled.
    #[prop(optional, into)]
    disabled_reason: Signal<String>,

    /// The label comes from [`MessageViewerTexts::view_message`].
    #[prop(optional)]
    texts: MessageViewerTexts,
) -> impl IntoView {
    view! {
        <RowActionButton
            kind=RowActionKind::Open
            name=name
            on_click=on_click
            disabled_reason=disabled_reason
            texts=view_message_row_texts(&texts)
        />
    }
}
