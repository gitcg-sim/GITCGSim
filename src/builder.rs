use crate::std_subset::{marker::PhantomData, Box};

use crate::zobrist_hash::ZobristHasher;
use crate::{cards::ids::*, data_structures::Vector, prelude::*, types::by_player::ByPlayer};

#[derive(Copy, Clone)]
pub enum StartingPhase {
    SelectStartingCharacter,
    RollPhase,
}

#[derive(Copy, Clone)]
pub struct StartingCondition {
    pub starting_phase: StartingPhase,
}

impl Default for StartingCondition {
    fn default() -> Self {
        Self {
            starting_phase: StartingPhase::RollPhase,
        }
    }
}

impl StartingCondition {
    pub fn new(starting_phase: StartingPhase) -> Self {
        Self { starting_phase }
    }

    pub fn starting_phase(&self) -> Phase {
        match self.starting_phase {
            StartingPhase::RollPhase => Phase::new_roll_phase(PlayerId::PlayerFirst),
            StartingPhase::SelectStartingCharacter => Phase::SelectStartingCharacter {
                state: Default::default(),
            },
        }
    }
}

macro_rules! typestate_trait {
    ($v: vis $Trait: ident { $($Type: ident),+ $(,)? }) => {
        $v trait $Trait {}
        $(
            #[derive(Default, Copy, Clone)]
            $v struct $Type;
            impl $Trait for $Type {}
        )+
    }
}

typestate_trait!(pub CharactersState { MissingCharacters, HasCharacters });
typestate_trait!(pub StartingConditionState { MissingStartingCondition, HasStartingCondition });

#[derive(Clone)]
pub struct GameStateInitializer<C: CharactersState, S: StartingConditionState> {
    pub characters: ByPlayer<Vector<CharId>>,
    pub starting_condition: StartingCondition,
    pub enable_log: bool,
    pub ignore_costs: bool,
    _marker: PhantomData<(C, S)>,
}

impl Default for GameStateInitializer<MissingCharacters, MissingStartingCondition> {
    fn default() -> Self {
        Self {
            characters: Default::default(),
            starting_condition: Default::default(),
            enable_log: Default::default(),
            ignore_costs: false,
            _marker: PhantomData,
        }
    }
}

impl GameStateInitializer<HasCharacters, MissingStartingCondition> {
    pub fn new<T: Into<Vector<CharId>>>(c1: T, c2: T) -> Self {
        Self {
            characters: ByPlayer::new(c1.into(), c2.into()),
            starting_condition: Default::default(),
            enable_log: Default::default(),
            ignore_costs: false,
            _marker: PhantomData,
        }
    }
}

// TODO starting dice/hands
impl<C: CharactersState, S: StartingConditionState> GameStateInitializer<C, S> {
    pub fn enable_log(self, enable_log: bool) -> Self {
        Self { enable_log, ..self }
    }

    pub fn ignore_costs(self, ignore_costs: bool) -> Self {
        Self { ignore_costs, ..self }
    }

    pub fn starting_condition(
        self,
        starting_condition: StartingCondition,
    ) -> GameStateInitializer<C, HasStartingCondition> {
        GameStateInitializer {
            characters: self.characters,
            starting_condition,
            enable_log: self.enable_log,
            ignore_costs: self.ignore_costs,
            _marker: PhantomData,
        }
    }

    pub fn start_at_select_character(self) -> GameStateInitializer<C, HasStartingCondition> {
        // TODO Select starting character broken?
        self.starting_condition(StartingCondition {
            starting_phase: StartingPhase::SelectStartingCharacter,
        })
    }

    pub fn skip_to_roll_phase(self) -> GameStateInitializer<C, HasStartingCondition> {
        self.starting_condition(StartingCondition {
            starting_phase: StartingPhase::RollPhase,
        })
    }

    pub fn characters<T: Into<Vector<CharId>>>(self, chars1: T, chars2: T) -> GameStateInitializer<HasCharacters, S> {
        GameStateInitializer {
            characters: (chars1.into(), chars2.into()).into(),
            starting_condition: self.starting_condition,
            enable_log: self.enable_log,
            ignore_costs: self.ignore_costs,
            _marker: PhantomData,
        }
    }
}

impl GameStateInitializer<HasCharacters, HasStartingCondition> {
    pub fn new_skip_to_roll_phase<T: Into<Vector<CharId>>>(c1: T, c2: T) -> Self {
        GameStateInitializer::<_, _>::new(c1, c2).starting_condition(StartingCondition::new(StartingPhase::RollPhase))
    }

    #[inline(always)]
    fn empty_game_state() -> GameState {
        GameState {
            players: ByPlayer::new(PlayerState::new([]), PlayerState::new([])),
            pending_cmds: None,
            phase: Phase::new_roll_phase(PlayerId::PlayerFirst),
            round_number: 1,
            ignore_costs: false,
            log: None,
            _incremental_hash: Default::default(),
            _hash: Default::default(),
        }
    }

    pub fn build(self) -> GameState {
        self.try_build().expect("Failed to build GameState.")
    }

    pub fn try_build(self) -> Option<GameState> {
        const RANGE: crate::std_subset::ops::RangeInclusive<usize> = 1..=8;
        if !RANGE.contains(&self.characters.get(PlayerId::PlayerFirst).len()) {
            return None;
        }
        if !RANGE.contains(&self.characters.get(PlayerId::PlayerSecond).len()) {
            return None;
        }

        let mut res = GameState {
            players: ByPlayer::new(
                PlayerState::new(self.characters.get(PlayerId::PlayerFirst).iter().copied()),
                PlayerState::new(self.characters.get(PlayerId::PlayerSecond).iter().copied()),
            ),
            phase: self.starting_condition.starting_phase(),
            ignore_costs: self.ignore_costs,
            ..Self::empty_game_state()
        };
        if self.enable_log {
            res.log = Some(Default::default());
        }
        res.rehash();
        Some(res)
    }
}

crate::with_updaters!(
    #[derive(Clone)]
    #[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
    pub struct GameStateBuilder {
        pub pending_cmds: Option<PendingCommands>,
        pub round_number: u8,
        pub phase: Phase,
        pub players: ByPlayer<PlayerState>,

        // Transient states
        #[cfg_attr(feature = "serde", serde(default))]
        pub log: Option<EventLog>,
        #[cfg_attr(feature = "serde", serde(default))]
        pub ignore_costs: bool,

        pub override_hash: Option<ZobristHasher>,
        pub override_incremental_hash: Option<ZobristHasher>,
    }
);

impl GameStateBuilder {
    pub fn new(players: ByPlayer<PlayerState>) -> Self {
        Self {
            pending_cmds: Default::default(),
            round_number: Default::default(),
            phase: Phase::SelectStartingCharacter {
                state: Default::default(),
            },
            players,
            log: Default::default(),
            ignore_costs: Default::default(),
            override_hash: Default::default(),
            override_incremental_hash: Default::default(),
        }
    }

    pub fn build(self) -> GameState {
        let should_rehash = self.override_hash.is_none() && self.override_incremental_hash.is_none();
        let mut gs = GameState {
            pending_cmds: self.pending_cmds.map(Box::new),
            round_number: self.round_number,
            phase: self.phase,
            players: self.players,
            log: self.log.map(Box::new),
            ignore_costs: self.ignore_costs,
            _hash: self.override_hash.unwrap_or_default(),
            _incremental_hash: self.override_incremental_hash.unwrap_or_default(),
        };
        if should_rehash {
            gs.rehash();
        }
        gs
    }
}

impl GameState {
    pub fn into_builder(self) -> GameStateBuilder {
        GameStateBuilder {
            pending_cmds: self.pending_cmds.map(|x| *x),
            round_number: self.round_number,
            phase: self.phase,
            players: self.players,
            log: self.log.map(|b| *b.clone()),
            ignore_costs: self.ignore_costs,
            override_hash: Some(self._hash),
            override_incremental_hash: Some(self._incremental_hash),
        }
    }
}

crate::impl_from_to_builder!(GameState, GameStateBuilder);

pub use crate::{
    dice_counter::builder::DiceCounterBuilder,
    types::{applied_effect_state::builder::AppliedEffectStateBuilder, char_state::builder::CharStateBuilder},
};

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_initializer_ignore_costs() {
        let game_state = GameStateInitializer::new(vec![CharId::Yoimiya], vec![CharId::Fischl])
            .ignore_costs(true)
            .starting_condition(StartingCondition::default())
            .build();
        assert!(game_state.ignore_costs);
    }

    #[test]
    fn test_initializer_enable_log() {
        let game_state = GameStateInitializer::new(vec![CharId::Yoimiya], vec![CharId::Fischl])
            .enable_log(true)
            .starting_condition(StartingCondition::default())
            .build();
        assert!(game_state.log.is_some());
    }
}
