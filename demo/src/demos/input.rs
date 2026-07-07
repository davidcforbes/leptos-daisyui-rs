use crate::core::{ContentLayout, Section};
use leptos::prelude::*;
use leptos_daisyui_rs::components::*;

#[component]
pub fn InputDemo() -> impl IntoView {
    let (value, set_value) = signal("".to_string());
    let (password, set_password) = signal("".to_string());
    let (numeric_value, set_numeric_value) = signal("".to_string());
    let (phone_value, set_phone_value) = signal("".to_string());
    let (search_value, set_search_value) = signal("".to_string());

    view! {
        <ContentLayout
            title="Input"
            description="Inputs are used to get user input in a text field"
        >
            <Section title="Basic Input">
                <Input
                    attr:placeholder="Type here"
                    attr:value=value
                    on:input=move |ev| set_value.set(event_target_value(&ev))
                    class="w-full max-w-xs"
                />
            </Section>

            <Section title="Colors">
                <div class="space-y-2">
                    <Input attr:placeholder="Default" class="w-full max-w-xs" />
                    <Input
                        color=InputColor::Primary
                        attr:placeholder="Primary"
                        class="w-full max-w-xs"
                    />
                    <Input
                        color=InputColor::Secondary
                        attr:placeholder="Secondary"
                        class="w-full max-w-xs"
                    />
                    <Input
                        color=InputColor::Accent
                        attr:placeholder="Accent"
                        class="w-full max-w-xs"
                    />
                    <Input color=InputColor::Info attr:placeholder="Info" class="w-full max-w-xs" />
                    <Input
                        color=InputColor::Success
                        attr:placeholder="Success"
                        class="w-full max-w-xs"
                    />
                    <Input
                        color=InputColor::Warning
                        attr:placeholder="Warning"
                        class="w-full max-w-xs"
                    />
                    <Input
                        color=InputColor::Error
                        attr:placeholder="Error"
                        class="w-full max-w-xs"
                    />
                </div>
            </Section>

            <Section title="Sizes">
                <div class="space-y-2">
                    <Input size=InputSize::Xs attr:placeholder="XS" class="w-full max-w-xs" />
                    <Input size=InputSize::Sm attr:placeholder="SM" class="w-full max-w-xs" />
                    <Input size=InputSize::Md attr:placeholder="MD" class="w-full max-w-xs" />
                    <Input size=InputSize::Lg attr:placeholder="LG" class="w-full max-w-xs" />
                </div>
            </Section>

            <Section title="Styles">
                <div class="space-y-2">
                    <Input attr:placeholder="Default" class="w-full max-w-xs" />
                    <Input attr:placeholder="Bordered" class="w-full max-w-xs" />
                    <Input
                        style=InputStyle::Ghost
                        attr:placeholder="Ghost"
                        class="w-full max-w-xs"
                    />
                </div>
            </Section>

            <Section title="Password with Reveal">
                <Input
                    input_type=InputType::Password
                    revealable=true
                    placeholder="Password"
                    value=password
                    on_input=move |v| set_password.set(v)
                    class="w-full max-w-xs"
                />
            </Section>

            <Section title="Numeric Filter">
                <Input
                    filter=InputFilter::Numeric
                    placeholder="Digits only"
                    value=numeric_value
                    on_input=move |v| set_numeric_value.set(v)
                    class="w-full max-w-xs"
                />
            </Section>

            <Section title="Phone Filter">
                <Input
                    filter=InputFilter::Phone
                    placeholder="(555) 123-4567"
                    value=phone_value
                    on_input=move |v| set_phone_value.set(v)
                    class="w-full max-w-xs"
                />
            </Section>

            <Section title="Search with Leading Icon">
                <Input
                    input_type=InputType::Search
                    placeholder="Search..."
                    value=search_value
                    on_input=move |v| set_search_value.set(v)
                    class="w-full max-w-xs"
                    leading_icon=Box::new(|| {
                        view! {
                            <svg
                                xmlns="http://www.w3.org/2000/svg"
                                class="h-4 w-4 opacity-50"
                                fill="none"
                                viewBox="0 0 24 24"
                                stroke="currentColor"
                            >
                                <path
                                    stroke-linecap="round"
                                    stroke-linejoin="round"
                                    stroke-width="2"
                                    d="M21 21l-6-6m2-5a7 7 0 11-14 0 7 7 0 0114 0z"
                                />
                            </svg>
                        }
                            .into_any()
                    })
                />
            </Section>
        </ContentLayout>
    }
}
