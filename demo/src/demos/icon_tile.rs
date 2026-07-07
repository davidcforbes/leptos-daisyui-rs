use crate::core::{ContentLayout, Section};
use leptos::prelude::*;
use leptos_daisyui_rs::components::*;

#[component]
pub fn IconTileDemo() -> impl IntoView {
    view! {
        <ContentLayout
            title="Icon Tile"
            description="A tinted rounded-square tile framing a centered icon glyph, with independent background and foreground colors."
        >
            <Section title="Colors" row=true>
                <IconTile bg=IconTileColor::Primary fg=IconTileColor::Primary>
                    <span>"P"</span>
                </IconTile>
                <IconTile bg=IconTileColor::Secondary fg=IconTileColor::Secondary>
                    <span>"S"</span>
                </IconTile>
                <IconTile bg=IconTileColor::Accent fg=IconTileColor::Accent>
                    <span>"A"</span>
                </IconTile>
                <IconTile bg=IconTileColor::Info fg=IconTileColor::Info>
                    <span>"I"</span>
                </IconTile>
                <IconTile bg=IconTileColor::Success fg=IconTileColor::Success>
                    <span>"S"</span>
                </IconTile>
                <IconTile bg=IconTileColor::Warning fg=IconTileColor::Warning>
                    <span>"W"</span>
                </IconTile>
                <IconTile bg=IconTileColor::Error fg=IconTileColor::Error>
                    <span>"E"</span>
                </IconTile>
            </Section>

            <Section title="Independent fg/bg" row=true>
                <IconTile bg=IconTileColor::Error fg=IconTileColor::Neutral>
                    <span>"!"</span>
                </IconTile>
                <IconTile bg=IconTileColor::Neutral fg=IconTileColor::Success>
                    <span>"\u{2713}"</span>
                </IconTile>
            </Section>

            <Section title="Sizes" row=true>
                <IconTile size=IconTileSize::Xs>
                    <span>"S"</span>
                </IconTile>
                <IconTile size=IconTileSize::Sm>
                    <span>"S"</span>
                </IconTile>
                <IconTile size=IconTileSize::Md>
                    <span>"S"</span>
                </IconTile>
                <IconTile size=IconTileSize::Lg>
                    <span>"S"</span>
                </IconTile>
                <IconTile size=IconTileSize::Xl>
                    <span>"S"</span>
                </IconTile>
            </Section>

            <Section title="Circle" row=true>
                <IconTile circle=true bg=IconTileColor::Primary fg=IconTileColor::Primary>
                    <span>"P"</span>
                </IconTile>
                <IconTile circle=true size=IconTileSize::Lg bg=IconTileColor::Accent fg=IconTileColor::Accent>
                    <span>"A"</span>
                </IconTile>
            </Section>
        </ContentLayout>
    }
}
