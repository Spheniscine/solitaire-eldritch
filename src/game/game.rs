use std::time::Duration;

use crate::game::{ATTACK_SLOTS_PER_MONSTER, NUM_SUITS, RANK_MAX};

pub const ANIMATION_DURATION: Duration = Duration::from_millis(200);
pub type AnimationKey = u16;

pub const MONSTER_RANK_START: u8 = 10;
pub const NUM_MONSTERS: usize = ((RANK_MAX + 1 - MONSTER_RANK_START) as usize) * NUM_SUITS;
pub const GRAVEYARD_TARGET: usize = NUM_MONSTERS * (ATTACK_SLOTS_PER_MONSTER + 1);