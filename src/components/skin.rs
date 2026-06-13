use dioxus::prelude::*;

use crate::{components::{Emoji, SkinTrait}, game::{Card, ColorMode, RankSkin, Skin, SuitSkin}};

pub const KATEX_MAIN: &str = "KaTeX_Main";

/// special rendering for rank 1 by putting a superscript 7 after it, as a reminder of the special ability in this game
#[component]
pub fn OneSupSeven() -> Element {
    rsx! {
        span {
            letter_spacing: "-0.05em",
            "1",
            span {
                font_size: "0.65em",
                position: "relative",
                top: "-0.5em",
                //left: "-0.1em",
                "7"
            }
        }
    }
}

impl SkinTrait<Card> for Skin {
    fn get_color(&self, card: &Card, mode: ColorMode) -> String {
        self.colors.color(card.suit, mode).to_string()
    }

    fn render_rank(&self, card: &Card) -> Element {
        rsx! {
            span {
                font_family: KATEX_MAIN,

                if self.ranks == RankSkin::Numbers && card.rank == 1 {
                    OneSupSeven {}
                } else {
                    {self.ranks.rank_text(card.rank)}
                }
            }
        }
    }

    fn render_suit(&self, card: &Card) -> Element {
        if self.suits == SuitSkin::Animals {
            rsx! {
                Emoji { 
                    text: self.suits.suit_symbol(card.suit)
                }
            }
        } else {
            rsx! {
                span {
                    font_family: self.suits.font(),
                    position: if self.suits == SuitSkin::Shapes || self.suits == SuitSkin::Mystique {"relative"},
                    top: if self.suits == SuitSkin::Shapes {"0.11em"}
                        else if self.suits == SuitSkin::Mystique {"-0.1em"},
                    font_weight: if self.suits == SuitSkin::Mystique {"bold"},
                    {self.suits.suit_symbol(card.suit)}
                }
            }
        }
    }
}