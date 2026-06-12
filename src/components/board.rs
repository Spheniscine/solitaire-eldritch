use dioxus::prelude::*;
use glam::Vec2;

use crate::{components::{CARD_BORDER_RADIUS_RATIO, CARD_HEIGHT_RATIO, CardComponent, CardFrame, rem}, game::{ATTACK_SLOTS_PER_MONSTER, AnimationKey, Board, BoardPos, Card, ColorMode, DepotRole, MONSTER_RANK_START, NUM_DEPOTS, Skin}};

#[component]
pub fn BoardComponent(
    position: Vec2,
    board: Board,
    skin: Skin,
    #[props(default)]
    onclick: EventHandler<BoardPos>,
    #[props(default)]
    ondoubleclick: EventHandler<BoardPos>,
    #[props(default)]
    oncontextmenu: EventHandler<BoardPos>,
    #[props(default)]
    animation_key: AnimationKey,
    #[props(default)]
    is_won: bool,
    #[props(default)]
    is_lost: bool,
) -> Element {
    let card_width = 11f32;
    let card_height = card_width * CARD_HEIGHT_RATIO;
    let spacer_x = 1f32;
    let spacer_y = 6f32;

    let pos_x = {
        let w = 8.;
        let left = 50. - (w * card_width + (w-1.) * spacer_x) / 2.;
        move |i: usize| {
            left + (card_width + spacer_x) * i as f32
        }
    };

    let start_y = 2f32;
    let pos_y = |i: usize| start_y + (card_height + spacer_y) * i as f32;
    let column_card_offset = Vec2::new(0., 6.);

    let get_pos = |depot: usize, ord: usize| {
        let (role, index) = DepotRole::role_and_subindex(depot).unwrap();
        match role {
            DepotRole::Tableau => 
                Vec2::new(pos_x(index), pos_y(0)) + column_card_offset * ord as f32,
            DepotRole::Monster => 
                Vec2::new(pos_x(5), pos_y(1 + index)),
            DepotRole::Attack => {
                let monster_slot_index = index / ATTACK_SLOTS_PER_MONSTER;
                let x_index = index % ATTACK_SLOTS_PER_MONSTER;

                Vec2::new(pos_x(6 + x_index), pos_y(1 + monster_slot_index))
            },
            DepotRole::Graveyard => Vec2::new(pos_x(7), pos_y(0)),
            DepotRole::Death => {
                Vec2::new(13., 13.) // todo
            },
        }
    };

    let get_hint = |depot: usize| {
        let role = DepotRole::role(depot).unwrap();
        match role {
            DepotRole::Tableau => Some(rsx!{}),
            DepotRole::Monster => 
                Some(
                    rsx!{
                        span {
                            font_family: "'Noto Emoji'",
                            "😈"
                        }
                    }
                ),
            DepotRole::Attack => 
                Some(
                    rsx!{
                        span {
                            font_family: "'Noto Emoji'",
                            "⚔"
                        }
                    }
                ),
            DepotRole::Graveyard => 
                Some(
                    rsx!{
                        span {
                            font_family: "'Noto Emoji'",
                            "🪦"
                        }
                    }
                ),
            DepotRole::Death => None,
        }
    };

    let is_face_up = |depot: usize| {
        DepotRole::role(depot).unwrap().is_face_up()
    };

    let selected_height = if let Some(BoardPos { depot_index, card_index }) = board.selected {
        let d = if DepotRole::role(depot_index).unwrap() == DepotRole::Tableau {
            board.depots[depot_index].len() - card_index - 1
        } else {
            0
        };

        card_height + column_card_offset.y * d as f32
    } else {0.};

    let anims = rsx!{}; // todo

    let color_mode = |card: Card| {
        if card.rank >= MONSTER_RANK_START {ColorMode::Dark} else {ColorMode::Light}
    };

    rsx! {
        div {
            position: "absolute",
            top: rem(position.y),
            left: rem(position.x),

            for depot in 0..NUM_DEPOTS {
                if let Some(hint) = get_hint(depot) {
                    CardFrame { 
                        position: get_pos(depot, 0),
                        width: card_width,
                        hint,
                        onclick: move |_| {
                            onclick.call(BoardPos::new(depot, !0))
                        },
                        oncontextmenu: move |ev: Event<MouseData>| {
                            ev.prevent_default();
                            oncontextmenu.call(BoardPos::new(depot, !0))
                        },
                    }
                }

                for i in 0..board.depots[depot].len() {
                    if board.selected == Some(BoardPos::new(depot, i)) {
                        div {
                            position: "absolute",
                            top: rem(get_pos(depot, i).y),
                            left: rem(get_pos(depot, i).x),
                            width: rem(card_width),
                            height: rem(selected_height),
                            background_color: "#ff0",
                            border_radius: rem(card_width * CARD_BORDER_RADIUS_RATIO),
                            class: "selected-halo",
                        }
                    }

                    CardComponent { 
                        position: get_pos(depot, i),
                        width: card_width,
                        card: if is_face_up(depot) {board.depots[depot][i]},
                        color_mode: color_mode(board.depots[depot][i]),
                        // number_hint: if !is_face_up(depot) {i + 1},
                        skin,
                        onclick: move |_| {
                            onclick.call(BoardPos::new(depot, i))
                        },
                        ondoubleclick: move |_| {
                            ondoubleclick.call(BoardPos::new(depot, i))
                        },
                        oncontextmenu: move |ev: Event<MouseData>| {
                            ev.prevent_default();
                            oncontextmenu.call(BoardPos::new(depot, i))
                        },
                    }
                }
            }

            {anims}

            if is_won {
                div {
                    position: "absolute",
                    top: rem(25.),
                    left: rem(17.5),
                    width: rem(59.),
                    background_color: "#505",
                    padding: rem(3.),
                    color: "#fff",
                    font_size: rem(7.),
                    border_radius: rem(2.),
                    text_align: "center",
                    "YOU WIN!",
                }
            }
        }
    }
}