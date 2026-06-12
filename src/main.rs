use dioxus::prelude::*;
use glam::Vec2;
use strum::IntoEnumIterator;

use crate::{components::CardComponent, game::{Card, Skin, Suit}};

mod game;
mod components;

const FAVICON: Asset = asset!("/assets/favicon.ico");

// altered version of KaTeX_Main to include filled "red" suits
const KATEX_SUITS: Asset = asset!("/assets/KaTeX_Suits.woff2");

const MAIN_CSS: Asset = asset!("/assets/main.css");
const HEADER_SVG: Asset = asset!("/assets/header.svg");

fn main() {
    dioxus::launch(App);
}

#[component]
fn App() -> Element {
    rsx! {
        document::Link {
            rel: "preconnect",
            href: "https://fonts.googleapis.com",
        }
        document::Link {
            rel: "preconnect",
            href: "https://fonts.gstatic.com",
            crossorigin: "anonymous",
        }
        document::Link {
            href: "https://fonts.googleapis.com/css2?family=Noto+Emoji:wght@300..700&family=Noto+Sans+Symbols+2&family=Noto+Sans+Symbols:wght@100..900&family=Noto+Sans:ital,wght@0,100..900;1,100..900&display=swap",
            rel: "stylesheet",
        }

        document::Link { rel: "icon", href: FAVICON }
        document::Link { rel: "stylesheet", href: MAIN_CSS }
        document::Style {
            r#"
            @font-face {{
                font-family: KaTeX_Main;
                font-style: normal;
                font-weight: 700;
                src: url({KATEX_SUITS}) format("woff2");
            }}
            "#,
        }
        Hero {}

    }
}

#[component]
pub fn Hero() -> Element {
    let skin = Skin::default();
    let iter = Suit::iter().enumerate();
    rsx! {
        div {
            id: "hero",
            for (i, suit) in iter.clone() {
                CardComponent { 
                    position: Vec2::new(10., 10. + 15. * i as f32),
                    width: 11.,
                    card: Card { rank: 13, suit, },
                    skin,
                    color_mode: game::ColorMode::Dark,
                }
            }

            for (i, suit) in iter {
                CardComponent { 
                    position: Vec2::new(30., 10. + 15. * i as f32),
                    width: 11.,
                    card: Card { rank: 1, suit, },
                    skin,
                    color_mode: game::ColorMode::Light,
                }
            }
        }
    }
}
