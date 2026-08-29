# Field association contract

`Field` owns the programmatic association between its visible label, its one
wrapped LDUI form control, and its current help or validation line. It assigns
a process-monotonic `ld-field-*` ID when the component is created; `Input`,
`Select`, and `Textarea` consume that ID through `FieldContext` without any
consumer call-site wiring.

The native allocator test proves sequential allocation. The real-WASM fixture
on `/components/fieldset` is the release contract: three Inputs and three
Selects render in one form, every ID is non-empty and unique, every `label[for]`
targets exactly one corresponding control, and every help/error reference
resolves exactly once. Browser console/WASM errors and blocking axe findings
also fail that journey.

Do not replace the generated ID from a child control or manually duplicate a
Field context. Raw controls may consume `FieldContext` directly, but they must
apply the same `id`, `aria-describedby`, `aria-errormessage`, and
`aria-invalid` contract.
