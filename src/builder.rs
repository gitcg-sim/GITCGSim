use crate::{types::by_player::ByPlayer, data_structures::Vector, cards::ids::*, prelude::*};

#[derive(Copy, Clone)]
pub enum StartingPhase {
    RollPhase
}

#[derive(Copy, Clone)]
pub struct StartingCondition {
    pub starting_phase: StartingPhase,
    // pub bypass_starting_char: bool,
}

impl Default for StartingCondition {
    fn default() -> Self {
        Self { starting_phase: StartingPhase::RollPhase }
    }
}

impl StartingCondition {
    pub fn starting_phase(&self) -> Phase {
        match self.starting_phase {
            StartingPhase::RollPhase => Phase::new_roll_phase(PlayerId::PlayerFirst),
        }
    }
}

#[derive(Clone)]
pub struct GameStateBuilder {
    pub chars: ByPlayer<Vector<CharId>>,
    pub starting_condition: StartingCondition,
    pub enable_log: bool,
    pub ignore_costs: bool,
}

impl GameStateBuilder {
    pub fn with_enable_log(self, enable_log: bool) -> Self {
        Self { enable_log, ..self }
    }

    pub fn with_ignore_costs(self, ignore_costs: bool) -> Self {
        Self { ignore_costs, ..self }
    }

    pub fn new<T: Into<Vector<CharId>>>(char_ids: ByPlayer<T>) -> Self {
        Self {
            chars: char_ids.map(|a| a.into()),
            starting_condition: Default::default(),
            enable_log: Default::default(),
            ignore_costs: false,
        }
    }

    #[inline(always)]
    fn empty_game_state() -> GameState {
        GameState {
            players: ByPlayer::new(PlayerState::new(Default::default()), PlayerState::new(Default::default())),
            pending_cmds: None,
            phase: Phase::new_roll_phase(PlayerId::PlayerFirst),
            round_number: 1,
            ignore_costs: false,
            log: Box::new(EventLog::new(false)),
            _incremental_hash: Default::default(),
            _hash: Default::default(),
        }
    }

    pub fn build(self) -> GameState {
        self.try_build().expect("Failed to build GameState.")
    }

    pub fn try_build(self) -> Option<GameState> {
        const RANGE: std::ops::RangeInclusive<usize> = 1..=8;
        if !RANGE.contains(&self.chars.get(PlayerId::PlayerFirst).len()) {
            return None
        }
        if !RANGE.contains(&self.chars.get(PlayerId::PlayerSecond).len()) {
            return None
        }

        let mut res = GameState {
            players: ByPlayer::new(
                PlayerState::new(self.chars.get(PlayerId::PlayerFirst)),
                PlayerState::new(self.chars.get(PlayerId::PlayerSecond))
            ),
            phase: self.starting_condition.starting_phase(),
            ignore_costs: self.ignore_costs,
            ..Self::empty_game_state()
        };
        res.log.enabled = self.enable_log;
        res.rehash();
        Some(res)
    }
}
