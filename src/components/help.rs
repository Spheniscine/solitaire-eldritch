use dioxus::prelude::*;

use crate::{components::{CardText, OneSupSeven, VIDEO_GAMEPLAY, rem, skin::KATEX_MAIN}, game::{Card, ColorMode, GameState, RankSkin, ScreenState, Suit}};

#[component]
fn Emph(children: Element) -> Element {
    rsx! {
        strong {
            color: "#ff0",
            {children}
        }
    }
}

#[component]
fn OhNo(children: Element) -> Element {
    rsx! {
        strong {
            color: "#f88",
            {children}
        }
    }
}

#[component]
pub fn Help(game_state: Signal<GameState>) -> Element {
    let st = game_state.read();
    let skin = st.skin;

    let stack_example = || {
        let mut ite = [
            Card { rank: 4, suit: Suit::Diamonds },
            Card { rank: 3, suit: Suit::Spades },
            Card { rank: 4, suit: Suit::Clubs },
            Card { rank: 3, suit: Suit::Hearts },
            Card { rank: 2, suit: Suit::Clubs },
        ].into_iter().map(|card| {
            rsx! {
                CardText { 
                    card, skin, color_mode: crate::game::ColorMode::Light,
                }
            }
        });


        let last = ite.next().unwrap();
        rsx! {
            {ite.next().unwrap()},
            for x in ite { "–", {x} },
            " can be placed on the ", {last}
        }
    };

    let attack_example = || {
        rsx! {
            "(e.g. ",
            CardText { card: Card { rank: 10, suit: Suit::Spades }, skin, color_mode: ColorMode::Light },
            " can be defeated by "

            CardText { card: Card { rank: 1, suit: Suit::Diamonds }, skin, color_mode: ColorMode::Light },
            " + "
            CardText { card: Card { rank: 9, suit: Suit::Clubs }, skin, color_mode: ColorMode::Light },
            
            ", or ",
            
            CardText { card: Card { rank: 1, suit: Suit::Spades }, skin, color_mode: ColorMode::Light },
            " + "
            CardText { card: Card { rank: 3, suit: Suit::Hearts }, skin, color_mode: ColorMode::Light },

            ")"
        }
    };

    let aces_text = if skin.ranks == RankSkin::Numbers {
        rsx! {
            "Cards with rank "
            span {
                font_family: KATEX_MAIN,
                font_size: "1.2em",
                OneSupSeven {  }
            },
            " (aces)"
        }
        
    } else {
        rsx! {
            "Aces"
        }
    };

    rsx! {
        div {
            style: "display: flex; flex-direction: column; align-items: center; font-size: 3.5rem; color: #fff; padding: 4rem;",
            class: "help",

            div {
                text_align: "left",

                p {
                    margin_top: "0",
                    "The deck is a standard 52-card deck with 13 ranks and 4 suits. Ranks ",
                    span {
                        font_family: KATEX_MAIN,
                        font_size: "1.2em",
                        "10"
                    },
                    " and above are evil ", Emph {"monsters"}, ". The rest are ", Emph {"attack cards"}, ".",
                }

                p {
                    "When a monster card is exposed in one of the 4 ", Emph {"tableau"}, " columns, it’ll jump into an open ",
                    Emph {"monster slot"}, ". If there are no open monster slots available, ", OhNo {"YOU LOSE"}, "."
                }

                p {
                    "Each monster slot has 2 ", Emph {"attack slots"}, " next to it. When the monster slot is filled, you may place 
                    attack cards in the attack slots. If the combined value of the 2 cards in the attack slots is ", Emph {"exactly equal"},
                    " to the value of the monster card, the monster is ", Emph {"defeated"}, ", and all 3 cards will go to the " Emph {"graveyard"}, "."
                }

                p {
                    {aces_text}, " are normally of value 1, but it has a special ability: if it’s in an attack slot for a monster of ", 
                    Emph {"matching suit"}, ", its value ",Emph {"becomes 7"},". NOTE: You must use this ability at least twice in a winning game."
                }

                p {
                    {attack_example()}
                }

                p {
                    "Attack cards in the tableau stack by ", Emph {"adjacent rank"}, " and " Emph {"unlike suit"}, ". Such stacks
                    of any size can be moved as a unit. (e.g. ",{stack_example()}")"
                }

                p {
                    "To ",Emph{"win the game"},", defeat all 16 monsters."
                }

                p {
                    Emph{"Shortcut note:"}," After selecting a stack, you may ", Emph {"right-click / long-press"}, " another tableau column to
                    stack in ", Emph {"reverse order"}, ". This shortcuts moving those cards one by one."
                }

                div {
                    position: "absolute",
                    bottom: rem(2.),
                    width: "92rem",
                    display: "flex",
                    justify_content: "center",

                    a {
                        href: VIDEO_GAMEPLAY,
                        target: "_blank",
                        text_decoration: "none",
                        margin_right: rem(4.),
                        div {
                            width: rem(30.),
                            position: "relative",
                            class: "game-button",
                            "Example video"
                        }
                    }

                    div {
                        width: rem(30.),
                        position: "relative",
                        class: "game-button",
                        onclick: move |_| game_state.write().screen_state = ScreenState::Game,
                        "Back to game"
                    }
                }
                
            }
        }
        
    }
}