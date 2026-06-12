use std::time::Duration;

use rand::{Rng, RngExt, seq::SliceRandom};
use serde::{Deserialize, Serialize};
use strum::IntoEnumIterator;

use crate::game::{ATTACK_SLOTS_PER_MONSTER, Board, BoardPos, Card, DECK_SIZE, DepotRole, NUM_SUITS, RANK_MAX, RANK_MIN, RANKS, RunsTrait, Skin, Suit};

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
        // "banded" generation algorithm that prevents clumps?
        let mut deck = Vec::with_capacity(DECK_SIZE);

        let mut attack_deck = Vec::with_capacity(DECK_SIZE - NUM_MONSTERS);
        let mut monster_deck = Vec::with_capacity(NUM_MONSTERS);

        for suit in Suit::iter() {
            for rank in RANK_MIN..MONSTER_RANK_START {
                attack_deck.push(Card { rank, suit });
            }
            for rank in MONSTER_RANK_START..=RANK_MAX {
                monster_deck.push(Card { rank, suit });
            }
        }

        attack_deck.shuffle(rng);
        monster_deck.shuffle(rng);
        let band_shift = rng.random_range(0..3);

        for i in 0..DECK_SIZE / DepotRole::Tableau.number_of() {
            for j in 0..DepotRole::Tableau.number_of() {
                let is_monster = i > 0 && (i + j) % 3 == band_shift;
                deck.push(if is_monster {monster_deck.pop().unwrap()} else {attack_deck.pop().unwrap()})
            }
        }
        
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

    pub fn new_game(&mut self) {
        let deal = Self::new_deal(&mut rand::rng());
        self.board = Board::from_deal(&deal);
        self.deal = deal;
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
        true // todo
    }

    pub fn can_move(&self, pos1: BoardPos, pos2: BoardPos) -> bool {
        if pos1.depot_index == pos2.depot_index { return false; }
        true // todo
    }

    pub fn can_rev_move(&self, pos1: BoardPos, pos2: BoardPos) -> bool {
        if pos1.depot_index == pos2.depot_index { return false; }
        true // todo
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