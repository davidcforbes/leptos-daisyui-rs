# Modal

A flexible modal dialog component for displaying content in an overlay with backdrop and customizable actions.

## Description

The Modal component creates an overlay dialog that can display forms, confirmations, images, or any content that needs to capture user attention. It supports various sizes, positioning, and interaction patterns.

## Props

| Prop | Type | Default | Description |
|------|------|---------|-------------|
| `open` | `Signal<bool>` | `false` | Whether the modal is open |
| `backdrop` | `Signal<bool>` | `false` | Render a `ModalBackdrop` for click-to-close |
| `on_close_request` | `Option<Callback<ModalCloseProposal>>` | `None` | Controlled-close proposal sink. Supplying it makes `open` the only thing that can close the dialog — see below |
| `texts` | `Signal<ModalTexts>` | default | Localized copy for the modal chrome (the backdrop's close control) |
| `label` | `MaybeProp<String>` | - | Accessible name (`aria-label`) |
| `labelled_by` | `MaybeProp<String>` | - | Id of the visible heading naming the dialog (`aria-labelledby`); takes precedence over `label` |
| `described_by` | `MaybeProp<String>` | - | Id of the describing element (`aria-describedby`) |
| `class` | `&'static str` | `""` | Additional CSS classes |
| `node_ref` | `NodeRef<Dialog>` | - | Reference to the `<dialog>` element |
| `children` | `Children` | - | Modal content |

## Controlled close (`ldui-e0fw`)

A native `<dialog>` can close itself, and it does not tell anyone in a way a
Leptos owner can see:

| Gesture | `cancel` fires? | `close` fires? |
|---|---|---|
| Escape | **yes** (cancelable) | yes, unless `cancel` was `preventDefault`ed |
| `ModalBackdrop` activation (a `method="dialog"` form submit) | **no** | yes |
| An in-content `<form method="dialog">` submit | **no** | yes |
| `dialog.close()` — what this component calls when `open` goes false | no | yes |

That table is the whole defect. `cancel`-based workarounds catch Escape only,
and `close` cannot tell a user dismissal from the owner's own programmatic
close. So a caller's `open` signal keeps reading `true` over a shut dialog;
a later `true`-to-`true` change is not a change and cannot reopen it, and
scoped feedback is never cleared.

Supplying `on_close_request` switches the dialog into **controlled** mode
(observable as `data-modal-close-mode="controlled"` on the `<dialog>`),
where the caller's signal is the only thing that ever closes it:

- Escape is vetoed on `cancel` and re-emitted as `ModalCloseCause::Escape`.
- A backdrop or in-content dialog-form submit is vetoed on `submit` (which
  bubbles to the dialog) and re-emitted as `ModalCloseCause::Backdrop` or
  `ModalCloseCause::DialogForm`. A form with any other `method` submits
  untouched, so a real search or login form inside the modal still works.
- **Accepting** a proposal means setting `open` to `false`. The component
  then calls the dialog's own `close()` exactly once.
- **Ignoring or rejecting** a proposal leaves the dialog open and the
  accepted state untouched. Nothing optimistic was written, so there is
  nothing to reconcile — the dialog never enters a state the caller did not
  ask for.
- A programmatic `open = false` emits **no** proposal. Proposals only ever
  originate from a user gesture.

Without `on_close_request` nothing changes: Escape and the backdrop close
natively and existing `on:close` call sites behave exactly as before.

```rust
let (open, set_open) = signal(false);
let (feedback, set_feedback) = signal(None::<String>);

view! {
    <Modal
        open=open
        backdrop=true
        labelled_by="reassign-title"
        on_close_request=Callback::new(move |proposal: ModalCloseProposal| {
            // A dismissal drops scoped feedback; a deliberate dialog-form
            // confirm keeps it.
            if proposal.cause != ModalCloseCause::DialogForm {
                set_feedback.set(None);
            }
            set_open.set(false);
        })
    >
        <ModalBox>
            <h3 id="reassign-title">"Reassign matter"</h3>
        </ModalBox>
    </Modal>
}
```

### Focus return

Trigger-focus restoration is the **platform's**, not this component's. A
modal opened with `show_modal()` records the previously focused element and
restores focus to it when `close()` runs. This component's only job is to
make sure every close really does go through `close()` — never through a
removed or hidden dialog — which is exactly what controlled mode guarantees.
Owning focus here would mean fighting that machinery and would break the
common case where the trigger has been re-rendered. A caller that wants
focus somewhere else moves it from `on_close_request`.

### Drift repair

If a `close` ever does reach a controlled dialog while the accepted target is
still `true`, the component re-shows it rather than reporting the
inconsistency. The accepted state is the truth; the DOM is made to match it.

## Subcomponents

### ModalBox
The main content container of the modal with proper spacing and layout.

| Prop | Type | Default | Description |
|------|------|---------|-------------|
| `class` | `&'static str` | `""` | Additional CSS classes |
| `children` | `Children` | - | Modal content |

### ModalAction
Action area for buttons and interactive elements, typically at the bottom.

| Prop | Type | Default | Description |
|------|------|---------|-------------|
| `class` | `&'static str` | `""` | Additional CSS classes |
| `children` | `Children` | - | Action elements |

### ModalBackdrop
daisyUI's click-to-close backdrop: a `method="dialog"` form covering the area
outside the modal box. Carries `data-modal-backdrop="true"` as a stable hook.

| Prop | Type | Default | Description |
|------|------|---------|-------------|
| `class` | `&'static str` | `""` | Additional CSS classes |
| `texts` | `Signal<ModalTexts>` | default | Label and accessible name of the close control (`"close"`) |
| `node_ref` | `NodeRef<Form>` | - | Reference to the `<form>` element |

## Examples

### Basic Modal

<details>
<summary>View Code</summary>

```rust
use leptos::prelude::*;
use leptos_daisyui_rs::components::*;

#[component]
fn BasicModal() -> impl IntoView {
    let (modal_open, set_modal_open) = signal(false);
    
    view! {
        <div>
            <Button 
                style=Signal::derive(|| ButtonStyle::Primary)
                on:click=move |_| set_modal_open.set(true)
            >
                "Open Modal"
            </Button>
            
            <Modal 
                open=Signal::derive(move || modal_open.get())
                on_close=Callback::new(move |_| set_modal_open.set(false))
            >
                <ModalBox class="w-11/12 max-w-5xl">
                    <h3 class="font-bold text-lg">"Hello World!"</h3>
                    <p class="py-4">"This is a basic modal dialog. You can put any content here."</p>
                    <ModalAction>
                        <Button 
                            style=Signal::derive(|| ButtonStyle::Primary)
                            on:click=move |_| set_modal_open.set(false)
                        >
                            "Close"
                        </Button>
                    </ModalAction>
                </ModalBox>
            </Modal>
        </div>
    }
}
```

</details>

### Confirmation Modal

<details>
<summary>View Code</summary>

```rust
use leptos::prelude::*;
use leptos_daisyui_rs::components::*;

#[component]
fn ConfirmationModal() -> impl IntoView {
    let (modal_open, set_modal_open) = signal(false);
    let (confirmed, set_confirmed) = signal(false);
    
    let handle_confirm = move |_| {
        set_confirmed.set(true);
        set_modal_open.set(false);
    };
    
    let handle_cancel = move |_| {
        set_modal_open.set(false);
    };
    
    view! {
        <div>
            <Button 
                color=Signal::derive(|| ButtonColor::Error)
                on:click=move |_| set_modal_open.set(true)
            >
                "Delete Item"
            </Button>
            
            {move || if confirmed.get() {
                view! {
                    <div class="alert alert-success mt-4">
                        <span>"Item deleted successfully!"</span>
                    </div>
                }.into_any()
            } else {
                view! { <div></div> }.into_any()
            }}
            
            <Modal 
                open=Signal::derive(move || modal_open.get())
                on_close=Callback::new(move |_| set_modal_open.set(false))
            >
                <ModalBox>
                    <h3 class="font-bold text-lg">"Confirm Deletion"</h3>
                    <p class="py-4">"Are you sure you want to delete this item? This action cannot be undone."</p>
                    <ModalAction class="justify-end space-x-2">
                        <Button 
                            style=Signal::derive(|| ButtonStyle::Ghost)
                            on:click=handle_cancel
                        >
                            "Cancel"
                        </Button>
                        <Button 
                            color=Signal::derive(|| ButtonColor::Error)
                            on:click=handle_confirm
                        >
                            "Delete"
                        </Button>
                    </ModalAction>
                </ModalBox>
            </Modal>
        </div>
    }
}
```

</details>

### Form Modal

<details>
<summary>View Code</summary>

```rust
use leptos::prelude::*;
use leptos_daisyui_rs::components::*;

#[component]
fn FormModal() -> impl IntoView {
    let (modal_open, set_modal_open) = signal(false);
    let (name, set_name) = signal("".to_string());
    let (email, set_email) = signal("".to_string());
    let (submitted, set_submitted) = signal(false);
    
    let handle_submit = move |_| {
        // Simulate form submission
        set_submitted.set(true);
        set_modal_open.set(false);
        // Reset form
        set_name.set("".to_string());
        set_email.set("".to_string());
    };
    
    view! {
        <div>
            <Button 
                style=Signal::derive(|| ButtonStyle::Primary)
                on:click=move |_| set_modal_open.set(true)
            >
                "Add User"
            </Button>
            
            {move || if submitted.get() {
                view! {
                    <div class="alert alert-success mt-4">
                        <span>"User added successfully!"</span>
                    </div>
                }.into_any()
            } else {
                view! { <div></div> }.into_any()
            }}
            
            <Modal 
                open=Signal::derive(move || modal_open.get())
                on_close=Callback::new(move |_| set_modal_open.set(false))
            >
                <ModalBox>
                    <h3 class="font-bold text-lg">"Add New User"</h3>
                    <div class="form-control w-full space-y-4 mt-4">
                        <div>
                            <label class="label">
                                <span class="label-text">"Name"</span>
                            </label>
                            <Input 
                                placeholder="Enter name"
                                value=Signal::derive(move || name.get())
                                on:input=move |ev| set_name.set(event_target_value(&ev))
                            />
                        </div>
                        <div>
                            <label class="label">
                                <span class="label-text">"Email"</span>
                            </label>
                            <Input 
                                placeholder="Enter email"
                                input_type=Signal::derive(|| InputType::Email)
                                value=Signal::derive(move || email.get())
                                on:input=move |ev| set_email.set(event_target_value(&ev))
                            />
                        </div>
                    </div>
                    <ModalAction class="justify-end space-x-2 mt-6">
                        <Button 
                            style=Signal::derive(|| ButtonStyle::Ghost)
                            on:click=move |_| set_modal_open.set(false)
                        >
                            "Cancel"
                        </Button>
                        <Button 
                            style=Signal::derive(|| ButtonStyle::Primary)
                            disabled=Signal::derive(move || name.get().is_empty() || email.get().is_empty())
                            on:click=handle_submit
                        >
                            "Add User"
                        </Button>
                    </ModalAction>
                </ModalBox>
            </Modal>
        </div>
    }
}
```

</details>

### Image Modal

<details>
<summary>View Code</summary>

```rust
use leptos::prelude::*;
use leptos_daisyui_rs::components::*;

#[component]
fn ImageModal() -> impl IntoView {
    let (modal_open, set_modal_open) = signal(false);
    
    view! {
        <div>
            <div class="cursor-pointer" on:click=move |_| set_modal_open.set(true)>
                <img 
                    src="https://via.placeholder.com/300x200" 
                    alt="Click to enlarge"
                    class="rounded-lg shadow-lg hover:shadow-xl transition-shadow"
                />
                <p class="text-sm text-gray-600 mt-2">"Click to enlarge"</p>
            </div>
            
            <Modal 
                open=Signal::derive(move || modal_open.get())
                on_close=Callback::new(move |_| set_modal_open.set(false))
            >
                <ModalBox class="w-11/12 max-w-5xl">
                    <div class="relative">
                        <button 
                            class="btn btn-sm btn-circle btn-ghost absolute right-2 top-2"
                            on:click=move |_| set_modal_open.set(false)
                        >
                            "✕"
                        </button>
                        <img 
                            src="https://via.placeholder.com/800x600" 
                            alt="Enlarged view"
                            class="w-full h-auto rounded-lg"
                        />
                    </div>
                    <div class="py-4">
                        <h3 class="font-bold text-lg">"Image Title"</h3>
                        <p>"This is a detailed view of the image with additional information."</p>
                    </div>
                </ModalBox>
            </Modal>
        </div>
    }
}
```

</details>

### Multi-step Modal

<details>
<summary>View Code</summary>

```rust
use leptos::prelude::*;
use leptos_daisyui_rs::components::*;

#[component]
fn MultiStepModal() -> impl IntoView {
    let (modal_open, set_modal_open) = signal(false);
    let (current_step, set_current_step) = signal(1);
    let (form_data, set_form_data) = signal(HashMap::new());
    
    let handle_next = move |_| {
        set_current_step.update(|step| *step += 1);
    };
    
    let handle_previous = move |_| {
        set_current_step.update(|step| *step -= 1);
    };
    
    let handle_finish = move |_| {
        // Process form data
        set_modal_open.set(false);
        set_current_step.set(1);
    };
    
    view! {
        <div>
            <Button 
                style=Signal::derive(|| ButtonStyle::Primary)
                on:click=move |_| set_modal_open.set(true)
            >
                "Start Setup"
            </Button>
            
            <Modal 
                open=Signal::derive(move || modal_open.get())
                backdrop_close=Signal::derive(|| false)
            >
                <ModalBox class="w-11/12 max-w-2xl">
                    <h3 class="font-bold text-lg">
                        {move || format!("Setup - Step {} of 3", current_step.get())}
                    </h3>
                    
                    // Progress indicator
                    <div class="flex justify-between items-center mt-4 mb-6">
                        {(1..=3).map(|step| {
                            let is_current = move || current_step.get() == step;
                            let is_completed = move || current_step.get() > step;
                            view! {
                                <div class=format!(
                                    "w-8 h-8 rounded-full flex items-center justify-center text-sm font-bold {}",
                                    if is_completed() || is_current() { "bg-primary text-primary-content" } else { "bg-base-300" }
                                )>
                                    {step}
                                </div>
                            }
                        }).collect::<Vec<_>>()}
                    </div>
                    
                    // Step content
                    <div class="py-4">
                        {move || match current_step.get() {
                            1 => view! {
                                <div>
                                    <h4 class="font-semibold mb-2">"Personal Information"</h4>
                                    <div class="space-y-4">
                                        <Input placeholder="Full Name" />
                                        <Input placeholder="Email Address" />
                                    </div>
                                </div>
                            }.into_any(),
                            2 => view! {
                                <div>
                                    <h4 class="font-semibold mb-2">"Preferences"</h4>
                                    <div class="space-y-4">
                                        <Checkbox>"Enable notifications"</Checkbox>
                                        <Checkbox>"Subscribe to newsletter"</Checkbox>
                                    </div>
                                </div>
                            }.into_any(),
                            3 => view! {
                                <div>
                                    <h4 class="font-semibold mb-2">"Review"</h4>
                                    <p>"Please review your information and click finish to complete setup."</p>
                                </div>
                            }.into_any(),
                            _ => view! { <div></div> }.into_any()
                        }}
                    </div>
                    
                    <ModalAction class="justify-between">
                        <div>
                            {move || if current_step.get() > 1 {
                                view! {
                                    <Button 
                                        style=Signal::derive(|| ButtonStyle::Ghost)
                                        on:click=handle_previous
                                    >
                                        "Previous"
                                    </Button>
                                }.into_any()
                            } else {
                                view! { <div></div> }.into_any()
                            }}
                        </div>
                        <div class="space-x-2">
                            <Button 
                                style=Signal::derive(|| ButtonStyle::Ghost)
                                on:click=move |_| {
                                    set_modal_open.set(false);
                                    set_current_step.set(1);
                                }
                            >
                                "Cancel"
                            </Button>
                            {move || if current_step.get() < 3 {
                                view! {
                                    <Button 
                                        style=Signal::derive(|| ButtonStyle::Primary)
                                        on:click=handle_next
                                    >
                                        "Next"
                                    </Button>
                                }.into_any()
                            } else {
                                view! {
                                    <Button 
                                        style=Signal::derive(|| ButtonStyle::Primary)
                                        on:click=handle_finish
                                    >
                                        "Finish"
                                    </Button>
                                }.into_any()
                            }}
                        </div>
                    </ModalAction>
                </ModalBox>
            </Modal>
        </div>
    }
}
```

</details>

## Accessibility

- Native `<dialog>` `show_modal()` focus trap and focus return to the trigger
- Escape key support for closing the modal — vetoed and re-proposed in
  controlled mode, so the dialog and the owner's state never disagree
- Name every dialog: `labelled_by` (preferred, points at the visible heading)
  or `label`. Without either, `aria-label="Modal"` is the floor, not the goal
- `described_by` for the summary paragraph under the heading
- Backdrop click to close (configurable via `backdrop`)

## Best Practices

1. Always provide a way to close the modal
2. Use appropriate modal sizes for content
3. Consider keyboard navigation and accessibility
4. Disable backdrop close for critical operations
5. Show loading states for async operations
6. Use confirmation patterns for destructive actions
7. When the caller owns `open`, wire `on_close_request` — do not hand-roll
   `on:cancel` plumbing, and never assume Escape or the backdrop left `open`
   alone
7. Keep modal content focused and concise