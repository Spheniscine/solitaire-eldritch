use std::{ops::RangeInclusive, time::Duration};

use rand::{Rng, seq::SliceRandom};
use serde::{Deserialize, Serialize};
use strum::IntoEnumIterator;

use crate::{components::LocalStorage, game::{ATTACK_SLOTS_PER_MONSTER, Board, BoardPos, Card, DECK_SIZE, DepotRole, NUM_SUITS, RANK_MAX, RANKS, RunsTrait, Skin, Suit}};

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

        //// testing death;
        // res.board.animation_acts.push(crate::game::AnimationAct::Move { 
        //     cards: vec![Card { rank: 13, suit: Suit::Hearts }], pos1: BoardPos::new(0, 13), 
        //     pos2: BoardPos::new(DepotRole::Death.id(0), 0), rev: false });

        res
    }

    fn is_misdeal(&self) -> bool {
        // attempts to filter out problematic or unwinnable deals

        const FRONT_MONSTER_RANGE: RangeInclusive<usize> = 1..=3;
        const MAX_MONSTER_CLUMP: usize = 3; 

        let front_monsters = DepotRole::Tableau.range()
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
        self.check_auto_moves();

        if !self.is_busy() { LocalStorage.save_game_state(&self); }
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

    pub fn can_stack(&self, back: Card, front: Card) -> bool {
        !back.is_monster() && 
            back.suit != front.suit && 
            back.rank.abs_diff(front.rank) == 1
    }

    pub fn can_select(&self, pos: BoardPos) -> bool {
        let depot = pos.depot_index;
        let ord = pos.card_index;

        if ord >= self.board.depots[depot].len() {
            return false;
        }
        let slice = &self.board.depots[depot][ord..];

        let Some(role) = DepotRole::role(depot) else { return false };
        match role {
            DepotRole::Tableau => slice.windows(2).all(|w| self.can_stack(w[0], w[1])),
            DepotRole::Monster => false,
            DepotRole::Attack => { slice.len() <= 1 },
            DepotRole::Graveyard => false,
            DepotRole::Death => false,
        }
    }

    pub fn can_move(&self, pos1: BoardPos, pos2: BoardPos) -> bool {
        if pos1.depot_index == pos2.depot_index { return false; }
        let depot1 = &self.board.depots[pos1.depot_index];
        let depot2 = &self.board.depots[pos2.depot_index];
        let num_moved = depot1.len() - pos1.card_index;
        if pos2.card_index != depot2.len() { return false; }
        
        let card = depot1[pos1.card_index];
        let Some((role, ix)) = DepotRole::role_and_subindex(pos2.depot_index) else { return false };
        match role {
            DepotRole::Tableau => depot2.last().is_none_or(|&c| self.can_stack(c, card)),
            DepotRole::Monster => false,
            DepotRole::Attack => {
                let monster_depot = &self.board.depots[DepotRole::Monster.id(ix / ATTACK_SLOTS_PER_MONSTER)];
                !monster_depot.is_empty() && depot2.is_empty() && num_moved == 1
            },
            DepotRole::Graveyard => false,
            DepotRole::Death => false,
        }
    }

    pub fn can_rev_move(&self, pos1: BoardPos, pos2: BoardPos) -> bool {
        DepotRole::role(pos2.depot_index) == Some(DepotRole::Tableau) &&
            self.can_move(self.board.last_pos(pos1.depot_index), pos2)
    }

    pub fn onclick(&mut self, pos: BoardPos) {
        if self.is_busy() { return; }
        if self.is_over() { return; }

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
        if self.is_over() { return; }

        if let Some(src) = self.board.selected {
            let dest = BoardPos { depot_index: pos.depot_index, card_index: pos.card_index.wrapping_add(1) };
            if !self.can_rev_move(src, dest) { return; }

            self.undo_stack.push(self.history.len());
            self.do_move_raw(src, dest, true);
        }
    }

    pub fn undo(&mut self) {
        if self.is_busy() || !self.undo_possible() { return; }
        let Some(target_len) = self.undo_stack.pop() else {return};
        while self.history.len() > target_len {
            let rec = self.history.pop().unwrap();
            self.board.do_move(rec.pos2, rec.pos1, rec.rev);
            self.board.advance_actions(); // no animation, as repeated card moves on same card causes problems
        }
        LocalStorage.save_game_state(&self);
    }

    pub fn restart(&mut self) {
        if self.history.is_empty() || !self.undo_possible() { return; }
        self.board = Board::from_deal(&self.deal);
        self.history.clear();
        self.undo_stack.clear();

        self.check_auto_moves();
        if !self.is_busy() { LocalStorage.save_game_state(&self); }
    }

    pub fn check_auto_moves(&mut self) {
        if self.is_busy() { return; }
        if self.is_over() { return; }
        
        // check if monsters are defeated
        for i in 0..DepotRole::Monster.number_of() {
            let dm = DepotRole::Monster.id(i);
            let da1 = DepotRole::Attack.id(i * ATTACK_SLOTS_PER_MONSTER);
            let da2 = DepotRole::Attack.id(i * ATTACK_SLOTS_PER_MONSTER + 1);

            let Some(&cm) = self.board.depots[dm].last() else {continue};
            let Some(&ca1) = self.board.depots[da1].last() else {continue};
            let Some(&ca2) = self.board.depots[da2].last() else {continue};

            // special power of aces: if match monster's suit, is worth 7
            let val = |c: Card| if c.rank == 1 && c.suit == cm.suit {7} else {c.rank};
            if val(ca1) + val(ca2) == cm.rank {
                let mut dest = self.board.top_pos(DepotRole::Graveyard.id(0));
                for d in [dm, da1, da2] {
                    self.do_move_raw(BoardPos::new(d, 0), 
                        dest, false);
                    dest.card_index += 1;
                }
                return;
            }
        }

        // check if monster is exposed
        for tableau in DepotRole::Tableau.range() {
            let Some(&card) = self.board.depots[tableau].last() else {continue};
            if card.is_monster() {
                let dest = DepotRole::Monster.range().find(|&d| {
                    self.board.depots[d].is_empty()
                }).unwrap_or(DepotRole::Death.id(0));
                self.do_move_raw(
                    self.board.last_pos(tableau),
                    self.board.top_pos(dest),
                    false
                );
                return;
            }
        }
    }

    pub fn is_won(&self) -> bool {
        self.board.depots[DepotRole::Graveyard.id(0)].len() == GRAVEYARD_TARGET
    }

    pub fn is_lost(&self) -> bool {
        !self.board.depots[DepotRole::Death.id(0)].is_empty()
    }

    pub fn is_over(&self) -> bool {
        self.is_won() || self.is_lost()
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

        if !self.is_busy() { LocalStorage.save_game_state(&self); }
    }
}