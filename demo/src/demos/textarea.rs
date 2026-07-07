use leptos::prelude::*;
use leptos_daisyui_rs::components::*;

const BIO_MAX_LEN: u32 = 160;

#[component]
pub fn TextareaDemo() -> impl IntoView {
    let (value, set_value) = signal("initial text".to_string());
    let char_count = move || value.get().chars().count();

    view! {
        <div class="space-y-6">
            <h1 class="text-3xl font-bold">"Textarea"</h1>
            <p class="text-base-content/70">"Textarea is used to get multi-line user input"</p>

            <div class="space-y-4">
                <h2 class="text-xl font-semibold">"Controlled Textarea"</h2>
                <p class="text-sm text-base-content/60">
                    "Bound to a signal via `value` / `on_input`; the character count below is derived from that same signal."
                </p>
                <Textarea
                    value=value
                    on_input=Callback::new(move |v: String| set_value.set(v))
                    placeholder="Bio"
                    maxlength=Some(BIO_MAX_LEN)
                    rows=Some(4u32)
                    class="w-full max-w-xs"
                />
                <div class="text-sm text-base-content/60">
                    {move || format!("{}/{} characters", char_count(), BIO_MAX_LEN)}
                </div>

                <h2 class="text-xl font-semibold">"Colors"</h2>
                <div class="space-y-2">
                    <Textarea placeholder="Default" class="w-full max-w-xs" />
                    <Textarea
                        color=TextareaColor::Primary
                        class="w-full max-w-xs"
                        disabled=true
                        placeholder="Primary"
                    />
                    <Textarea
                        color=TextareaColor::Secondary
                        placeholder="Secondary"
                        class="w-full max-w-xs"
                    />
                    <Textarea
                        color=TextareaColor::Accent
                        placeholder="Accent"
                        class="w-full max-w-xs"
                    />
                </div>

                <h2 class="text-xl font-semibold">"Sizes"</h2>
                <div class="space-y-2">
                    <Textarea size=TextareaSize::Xs placeholder="XS" class="w-full max-w-xs" />
                    <Textarea size=TextareaSize::Sm placeholder="SM" class="w-full max-w-xs" />
                    <Textarea size=TextareaSize::Md placeholder="MD" class="w-full max-w-xs" />
                    <Textarea size=TextareaSize::Lg placeholder="LG" class="w-full max-w-xs" />
                </div>

                <h2 class="text-xl font-semibold">"Form attributes"</h2>
                <div class="space-y-2">
                    <Textarea
                        name=Some("notes".to_string())
                        required=true
                        placeholder="Required field"
                        class="w-full max-w-xs"
                    />
                    <Textarea
                        readonly=true
                        value=Signal::derive(|| "Read-only content".to_string())
                        class="w-full max-w-xs"
                    />
                </div>
            </div>
        </div>
    }
}
