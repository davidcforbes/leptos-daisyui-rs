# MessageViewer

A read-only, opinionated view of a **sent** WhatsApp, SMS or MMS message as
the recipient saw it (ldui-mhur). Built for history screens; its first
consumer is 4iiz Office's Mass Communications campaign history.

It composes existing pieces and adds no second bubble renderer:
`RichMessageBubble` / `RichMessageThread` (ldui-g8hh) for the message and the
replies, `MockupPhone` for the frame, `Modal` for the dialog and the enlarged
picture, `Badge`, `Tooltip`, `Join` + `Button` for the switches, and
`RowActionButton` for the table action.

## Parts

| Item | What it is |
| --- | --- |
| `MessageViewer` | The phone frame (`frame=false` for a plain box), channel chrome, the bubble, the switches, the details list and the reply thread. |
| `MessageViewerDialog` | The viewer in the house `Modal`: visible title (its accessible name), close button, Escape and backdrop, all through `on_close` (controlled). |
| `ViewMessageRowAction` | The EntityTable eye action (`RowActionKind::Open`), named "View message: {name}" / "Ver mensaje: {name}". |
| `MessageViewerData` | Plain owned data: channel, sender, versions, `text_only_variant`, sent time, `DeliveryStatus` + reason, `MessageCounts`, replies. |
| `MessageViewerTexts` | Every string, `Default` English, `es()` Spanish. Nests `RichMessageTexts` as `rich`. |

## Behaviour

- **Chrome.** WhatsApp shows the business name and number with a
  **Template** badge (`whatsapp_template: true`, sent outside the 24-hour
  window) or **Session message**. SMS/MMS shows the sending number, an
  **MMS** badge when the shown message has a picture, and the bubble's
  segment indicator. An SMS-family message is MMS exactly when the shown copy
  carries a picture, so the text-only copy of an MMS campaign reads as SMS.
- **Picture.** Above the text, with a loading skeleton (`data-media-loading`,
  the header `aria-busy`) until it loads, a fallback tile saying "The picture
  could not be shown" when it fails, and a click-to-enlarge button
  (`data-message-image-open`) opening a nested dialog. A picture without alt
  text gets "{channel} picture" ("MMS picture", "Imagen de MMS").
- **Switches**, shown only for what the data supplies: language (two or more
  `MessageVersion`s), "With picture / Text only" (`text_only_variant` and a
  picture on the shown version), "As sent / Template" (the version has a
  `template`; placeholders render as rich_message's chips). Each switch is a
  `role="group"` of `aria-pressed` buttons; pressing one never rebuilds the
  button that has focus. Opening another message resets to the first
  version, with picture, as sent.
- **Details.** Channel, sent time, sender, a delivery-status chip from
  `DeliveryStatus` (Queued, Sent, Delivered, Read, Failed, NoReceipt) whose
  reason is its tooltip and `aria-describedby` (the chip is then focusable),
  and the reach line "Sent to 132 · with picture 120 · text only 12".
- **Replies** render as received bubbles under the sent one.

The component fetches nothing but the picture URLs the host passes and reads
no clock; times arrive as display strings.

## RichMessageBubble additions (opt-in)

Three new props, all off by default, so existing bubbles render exactly as
before: `image_placeholder` (the skeleton), `on_image_open` (the picture
becomes a `data-pressable` button reporting its `MessageHeader`) and
`image_open_label` (the button's name prefix; `image_open_label()` builds
"{prefix}: {alt}").

## From an EntityTable

```rust
let viewing = RwSignal::new(None::<(String, MessageViewerData)>);
let column = EntityColumn::action("view", "Message", |_: &Campaign| String::new())
    .render_with(move |row: &Campaign| {
        let name = row.name.clone();
        let label = name.clone();
        let data = row.viewer_data();
        view! {
            <EntityRowAction action_id="view-message">
                <ViewMessageRowAction
                    name=label
                    on_click=Callback::new(move |()| viewing.set(Some((name.clone(), data.clone()))))
                />
            </EntityRowAction>
        }
        .into_any()
    });

view! {
    <MessageViewerDialog
        open=Signal::derive(move || viewing.with(Option::is_some))
        on_close=Callback::new(move |()| viewing.set(None))
        title=Signal::derive(move || viewing.with(|v| v.as_ref().map(|(n, _)| n.clone()).unwrap_or_default()))
        data=Signal::derive(move || viewing.with(|v| v.as_ref().map(|(_, d)| d.clone()).unwrap_or_default()))
    />
}
```

Hoist row values into `let`s before `view!` (the macro captures `row` in a
deferred closure), and keep a columns `Vec` that sits inside `Section`-style
`Children` in a `StoredValue::new_local`.

## DOM hooks

`data-message-viewer` (root, with `data-viewer-channel` and
`data-viewer-mode`), `data-viewer-chrome`, `data-viewer-sender`,
`data-viewer-number`, `data-viewer-badge`, `data-viewer-screen`,
`data-viewer-switches`, `data-viewer-switch="language|picture|content"`,
`data-viewer-option`, `data-viewer-details`, `data-delivery-status`,
`data-delivery-status-reason`, `data-viewer-counts`, `data-viewer-sent-at`,
`data-viewer-replies`, `dialog[data-viewer-lightbox]`,
`data-viewer-lightbox-image`, `dialog[data-message-viewer-dialog]`,
`data-viewer-title`, `data-viewer-close`; on the bubble,
`data-message-image-open` and `data-media-loading`.

## Proof

Native: `src/components/message_viewer/tests.rs`. Browser:
`cargo xtask test-message-viewer` (`tests/message_viewer_smoke.rs`) over the
`/components/message_viewer` showcase.
