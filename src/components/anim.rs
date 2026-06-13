use dioxus::prelude::*;
use glam::Vec2;

use crate::{components::{Emoji, rem}, game::ANIMATION_DURATION};

#[component]
pub fn Movement(
    src_translate_vec: Vec2,
    children: Element,
) -> Element {
    rsx! {
        div {
            style: "--translateX: {rem(src_translate_vec.x)}; --translateY: {rem(src_translate_vec.y)}; 
            animation: {ANIMATION_DURATION.as_secs_f32()}s movement;",
            {children},
        }
    }
}

#[component]
pub fn TranslateVars(
    src_translate_vec: Vec2,
    children: Element,
) -> Element {
    rsx! {
        div {
            style: "--translateX: {rem(src_translate_vec.x)}; --translateY: {rem(src_translate_vec.y)};",
            {children},
        }
    }
}

#[component]
pub fn Collision(
    position: Vec2,
    size: f32,
) -> Element {
    rsx! {
        div {
            position: "absolute",
            left: rem(position.x),
            top: rem(position.y),
            class: "fading",
            font_size: rem(size),
            z_index: 10,

            Emoji { text: "💥" }
        }
    }
}