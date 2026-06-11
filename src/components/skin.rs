use dioxus::prelude::*;

use crate::{components::{Emoji, SkinTrait}, game::{Card, ColorMode, Skin, SuitSkin}};

pub const KATEX_MAIN: &str = "KaTeX_Main";

impl SkinTrait<Card> for Skin {
    fn get_color(&self, card: &Card, mode: ColorMode) -> String {
        self.colors.color(card.suit, mode).to_string()
    }

    fn render_rank(&self, card: &Card) -> Element {
        rsx! {
            span {
                font_family: KATEX_MAIN,
                {self.ranks.rank_text(card.rank)}
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
                    position: if self.suits == SuitSkin::Shapes || self.suits == SuitSkin::Mystical {"relative"},
                    top: if self.suits == SuitSkin::Shapes {"0.11em"}
                        else if self.suits == SuitSkin::Mystical {"-0.105em"},
                    font_weight: if self.suits == SuitSkin::Mystical {"bold"},
                    {self.suits.suit_symbol(card.suit)}
                }
            }
        }
    }
}