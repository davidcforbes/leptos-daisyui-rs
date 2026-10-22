# RichMessage (SMS, MMS and WhatsApp)

`RichMessageBubble`, `RichMessageThread` and `SmsSegmentIndicator`
(`src/components/rich_message/`, ldui-g8hh) render a message the way the
recipient sees it. They were built for 4iiz-Office's Mass Communications
template preview and the Account > Conversations thread, and they are generic:
nothing names a provider.

## The model

Hosts map their own shapes (Meta template components, Zoho Desk canned
messages, CTM SMS/MMS, stored conversation rows) into one neutral
`RichMessage`:

| Field | Meaning |
|---|---|
| `channel` | `Sms`, `Mms` or `Whatsapp`; picks the text renderer |
| `direction` | `In` (start), `Out` (end, neutral bubble: the dark theme's primary fails AA for body text) or `Preview` (start, inert buttons, placeholders recognised) |
| `author`, `time` | display strings; the component never reads a clock (no `std::time` in Wasm) |
| `header` | `Text`, `Image { src, alt, width, height }`, `Video { src, poster, alt }` or `Document { name, src }` |
| `body` | message text; line breaks, emoji and non-Latin text kept as given |
| `footer` | the small line under the body |
| `actions` | `Url`, `Call`, `CopyCode`, `QuickReply` |
| `quick_replies` | extra quick-reply labels, rendered after `actions` |
| `truncated` | `MessageTruncation { shown_len, true_len }` shows "Showing 4000 of 5210 characters" |

## Behaviour

- **Media.** The image keeps its aspect ratio inside the bubble. A load
  failure (or an empty `src`) swaps it for a fallback tile that reads "Image
  not available" plus the alt text or file name. The component fetches only
  the `src` it is given; a video never preloads.
- **WhatsApp formatting.** Only `*bold*`, `_italic_`, `~strike~` and
  triple-backtick monospace, by WhatsApp's own boundary rules: a marker opens
  after whitespace, punctuation or the start of the text, and never inside a
  word (`snake_case` stays literal). Markers never cross a line break, and
  monospace may span lines. SMS bodies are never formatted.
- **Placeholders.** `{{name}}`, `{{1}}` and `${Contacts.First Name}` are
  recognised in a preview, or whenever the host passes `placeholders`. A key
  with a non-blank value renders as a highlighted `<mark>`. Any other key
  renders as an outline chip with a screen-reader "unfilled" suffix.
  Placeholders are tokenised before formatting, so the underscore in
  `{{first_name}}` never opens italics.
- **Buttons.** Every action is a `<button class="btn btn-sm">` with no
  `href`. Without `on_action` it is a focusable `aria-disabled` preview,
  described by a hint ("Preview only: this is the recipient's button") and
  titled with its target (URL, number or code). With `on_action`, a press
  reports the `MessageAction` and the host decides what happens.
- **Links.** Off by default. `linkify=true` turns `http(s)://` URLs, `www.`
  hosts and email addresses into new-tab links (`rel="noopener noreferrer"`)
  and leaves trailing punctuation outside the link.
- **SMS indicator.** An SMS or MMS preview shows "GSM-7 · 2 segments · 161
  chars", "UCS-2 · 11 segments · 712 chars" or "Picture message · 712 chars".
  Counting follows 3GPP TS 23.038: 160/153 septets, with extension characters
  costing two; 70/67 UTF-16 units. Escape pairs and surrogate pairs never
  straddle a segment, and `chars` counts grapheme clusters.
  `sms_indicator_override` replaces the text when a carrier's limits differ,
  and `show_sms_indicator` forces it on or off. `SmsSegmentIndicator` is the
  same line for a composer and is a polite live region.
- **Not sendable.** `sendable=false` adds a warning badge, for a template the
  host refuses to send (Office refuses media-header WhatsApp sends).

## Accessibility and i18n

Each message is an `<article>` named "direction, author, time" ("Received,
+1 707-415-3915, 4:41 PM"). A thread is a labelled `<ol>`. Every string comes
from `RichMessageTexts` (`Default` is English, `RichMessageTexts::es()` is
Spanish), and a host overrides single fields with struct-update syntax.
Colours are daisyUI tokens only: the bubble's own colours, `bg-warning` for
filled placeholders, and `currentColor` outlines.

`cargo xtask test-rich-message` proves all of this in a real browser. It
checks every showcase section with axe (WCAG 2.2 AA through `tests/common/axe.rs`)
in both the light and dark themes, and runs the drift audit on the page.

## Pure helpers

`sms_segments`, `sms_indicator_text`, `message_label`, `truncation_note`,
`media_fallback_name` and `plan_action_button` are plain functions, and
`rich_message::inline::{split_placeholders, inline_runs}` expose the text
pipeline, so a host can reuse the counting or the parsing without rendering.
