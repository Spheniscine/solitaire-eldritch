use std::{ops::RangeInclusive, time::Duration};

use rand::{Rng, seq::SliceRandom};
use serde::{Deserialize, Serialize};
use strum::IntoEnumIterator;

use crate::game::{ATTACK_SLOTS_PER_MONSTER, Board, BoardPos, Card, DECK_SIZE, DepotRole, NUM_SUITS, RANK_MAX, RANKS, RunsTrait, Skin, Suit};

pub const ANIMATION_DURATION: Duration = Duration::from_millis(200);
pub type AnimationKey = u16;

pub const MONSTER_RANK_START: u8 = 10;
pub const NUM_MONSTERS: usize = ((RANK_MAX + 1 - MONSTER_RANK_START) as usize) * NUM_SUITS;
pub const GRAVEYARD_TARGET: usize = NUM_MONSTERS * (ATTACK_SLOTS_PER_MONSTER + 1);

impl Card {
    pub fn is_monster(self) -> bool {
        self.rank >= MONSTER_RANK_START
    }
}

#[derive(Clone, Serialize, Deserialize, Debug, PartialEq, Eq)]
pub struct ActionRecord {
    pos1: BoardPos, pos2: BoardPos, rev: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, Default)]
pub enum ScreenState {
    #[default] Game, 
    Settings, Help,
}

#[derive(Clone, Serialize, Deserialize, Debug, PartialEq, Eq)]
pub struct GameState {
    pub board: Board,
    pub deal: Vec<Card>,
    #[serde(skip)]
    pub animation_key: AnimationKey, // used for syncing and to provide animator components with cycling keys
    pub history: Vec<ActionRecord>,
    pub undo_stack: Vec<usize>,
    pub already_won: bool,
    pub num_wins: i32,

    pub screen_state: ScreenState,

    pub allow_undo: bool,
    pub skin: Skin,
}

impl GameState {
    pub fn new_deal(rng: &mut impl Rng) -> Vec<Card> {
        let mut deck = Vec::with_capacity(DECK_SIZE);
        for rank in RANKS {
            for suit in Suit::iter() {
                deck.push(Card { rank, suit });
            }
        }

        deck.shuffle(rng);
        deck
    }

    pub fn init() -> Self {
        let mut res = Self {
            board: Board::empty(),
            deal: vec![],
            animation_key: 0,
            history: vec![],
            undo_stack: vec![],
            already_won: false,
            num_wins: 0,
            screen_state: ScreenState::Game,
            allow_undo: true,
            skin: Skin::default(),
        };

        res.new_game();
        res
    }

    fn is_misdeal(&self) -> bool {
        // attempts to filter out problematic or unwinnable deals

        const FRONT_MONSTER_RANGE: RangeInclusive<usize> = 1..=3;
        const MAX_MONSTER_CLUMP: usize = 3; 

        let mut front_monsters = DepotRole::Tableau.range()
            .flat_map(|d| {
                self.board.depots[d].iter().rev().take_while(|&&c| c.is_monster())
            }).count();
        // dioxus::logger::tracing::debug!("Front monsters: {}", front_monsters);
        if !FRONT_MONSTER_RANGE.contains(&front_monsters) { return true; }

        let mut runs = DepotRole::Tableau.range()
            .flat_map(|d| {
                self.board.depots[d].iter().map(|&c| c.is_monster()).runs()
            });
        if runs.any(|(is_monster, len)| is_monster && len > MAX_MONSTER_CLUMP) {
            return true;
        }

        false
    }

    pub fn new_game(&mut self) {
        loop {
            let deal = Self::new_deal(&mut rand::rng());
            self.board = Board::from_deal(&deal);
            self.deal = deal;
            if !self.is_misdeal() { break; }
        }
        self.history.clear();
        self.undo_stack.clear();
        self.already_won = false;
        // LocalStorage.save_game_state(&self);
    }

    pub fn is_busy(&self) -> bool {
        self.is_acting()
    }

    pub fn is_acting(&self) -> bool {
        !self.board.animation_acts.is_empty()
    }

    pub fn undo_possible(&self) -> bool {
        self.allow_undo && !self.undo_stack.is_empty()
    }

    fn do_move_raw(&mut self, pos1: BoardPos, pos2: BoardPos, rev: bool) {
        self.board.do_move(pos1, pos2, rev);
        self.history.push(ActionRecord { pos1, pos2, rev })
    }

    pub fn can_select(&self, pos: BoardPos) -> bool {
        let depot = pos.depot_index;
        let ord = pos.card_index;

        if ord >= self.board.depots[depot].len() {
            return false;
        }

        true // todo
    }

    pub fn can_move(&self, pos1: BoardPos, pos2: BoardPos) -> bool {
        if pos1.depot_index == pos2.depot_index { return false; }
        let depot1 = &self.board.depots[pos1.depot_index];
        let depot2 = &self.board.depots[pos2.depot_index];
        let num_moved = depot1.len() - pos1.card_index;
        if pos2.card_index != depot2.len() { return false; }
        
        true // todo
    }

    pub fn can_rev_move(&self, pos1: BoardPos, pos2: BoardPos) -> bool {
        self.can_move(pos1, pos2) // todo
    }

    pub fn onclick(&mut self, pos: BoardPos) {
        if self.is_busy() { return; }

        if let Some(src) = self.board.selected {
            if pos == src { 
                self.board.selected = None; 
                return;
            }
            if src.depot_index == pos.depot_index && self.can_select(pos) {
                self.board.selected = Some(pos);
                return;
            }

            let dest = BoardPos { depot_index: pos.depot_index, card_index: pos.card_index.wrapping_add(1) };
            if !self.can_move(src, dest) { return; }

            self.undo_stack.push(self.history.len());
            self.do_move_raw(src, dest, false);
        } else {
            if self.can_select(pos) {
                self.board.selected = Some(pos);
            }
        }
    }

    // right-click is shortcut for reverse-stacking
    pub fn oncontextmenu(&mut self, pos: BoardPos) {
        if self.is_busy() { return; }

        if let Some(src) = self.board.selected {
            let dest = BoardPos { depot_index: pos.depot_index, card_index: pos.card_index.wrapping_add(1) };
            if !self.can_rev_move(src, dest) { return; }

            self.undo_stack.push(self.history.len());
            self.do_move_raw(src, dest, true);
        }
    }

    pub fn check_auto_moves(&mut self) {
        if self.is_busy() { return; }
        //todo
    }

    pub fn is_won(&self) -> bool {
        false
        //todo
    }

    pub fn advance_animations(&mut self, key: AnimationKey) {
        if key != self.animation_key { return; }
        self.animation_key = self.animation_key.wrapping_add(1);
        
        self.board.advance_actions();

        if self.is_won() {
            if !self.already_won {
                self.num_wins += 1;
                self.already_won = true;
            }
        } else {
            self.check_auto_moves();
        }

        // if !self.is_busy() { LocalStorage.save_game_state(&self); }
    }
}