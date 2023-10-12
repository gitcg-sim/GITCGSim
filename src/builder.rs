use std::marker::PhantomData;

use crate::{cards::ids::*, data_structures::Vector, prelude::*, types::by_player::ByPlayer};

#[derive(Copy, Clone)]
pub enum StartingPhase {
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
pub struct GameStateBuilder<C: CharactersState, S: StartingConditionState> {
    pub characters: ByPlayer<Vector<CharId>>,
    pub starting_condition: StartingCondition,
    pub enable_log: bool,
    pub ignore_costs: bool,
    _marker: PhantomData<(C, S)>,
}

impl Default for GameStateBuilder<MissingCharacters, MissingStartingCondition> {
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

impl GameStateBuilder<HasCharacters, MissingStartingCondition> {
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

impl<C: CharactersState, S: StartingConditionState> GameStateBuilder<C, S> {
    pub fn with_enable_log(self, enable_log: bool) -> Self {
        Self { enable_log, ..self }
    }

    pub fn with_ignore_costs(self, ignore_costs: bool) -> Self {
        Self { ignore_costs, ..self }
    }

    pub fn with_starting_condition(
        self,
        starting_condition: StartingCondition,
    ) -> GameStateBuilder<C, HasStartingCondition> {
        GameStateBuilder {
            characters: self.characters,
            starting_condition,
            enable_log: self.enable_log,
            ignore_costs: self.ignore_costs,
            _marker: PhantomData,
        }
    }

    pub fn with_characters<T: Into<Vector<CharId>>>(
        self,
        characters: ByPlayer<T>,
    ) -> GameStateBuilder<HasCharacters, S> {
        GameStateBuilder {
            characters: characters.map(|x| x.into()),
            starting_condition: self.starting_condition,
            enable_log: self.enable_log,
            ignore_costs: self.ignore_costs,
            _marker: PhantomData,
        }
    }
}

impl GameStateBuilder<HasCharacters, HasStartingCondition> {
    pub fn new_roll_phase(c1: &Vector<CharId>, c2: &Vector<CharId>) -> Self {
        GameStateBuilder::<_, _>::new(c1.clone(), c2.clone())
            .with_starting_condition(StartingCondition::new(StartingPhase::RollPhase))
    }

    pub fn new_roll_phase_1<T: Into<Vector<CharId>>>(c1: T, c2: T) -> Self {
        GameStateBuilder::<_, _>::new(c1, c2).with_starting_condition(StartingCondition::new(StartingPhase::RollPhase))
    }

    #[inline(always)]
    fn empty_game_state() -> GameState {
        GameState {
            players: ByPlayer::new(
                PlayerState::new(Default::default()),
                PlayerState::new(Default::default()),
            ),
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
        if !RANGE.contains(&self.characters.get(PlayerId::PlayerFirst).len()) {
            return None;
        }
        if !RANGE.contains(&self.characters.get(PlayerId::PlayerSecond).len()) {
            return None;
        }

        let mut res = GameState {
            players: ByPlayer::new(
                PlayerState::new(self.characters.get(PlayerId::PlayerFirst)),
                PlayerState::new(self.characters.get(PlayerId::PlayerSecond)),
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

#[cfg(test)]
mod tests {
    use crate::cards::ids::CharId;

    use super::*;

    #[test]
    fn test_builder_ignore_costs() {
        let game_state = GameStateBuilder::new(vec![CharId::Yoimiya], vec![CharId::Fischl])
            .with_ignore_costs(true)
            .with_starting_condition(StartingCondition::default())
            .build();
        assert!(game_state.ignore_costs);
    }

    #[test]
    fn test_builder_enable_log() {
        let game_state = GameStateBuilder::new(vec![CharId::Yoimiya], vec![CharId::Fischl])
            .with_enable_log(true)
            .with_starting_condition(StartingCondition::default())
            .build();
        assert!(game_state.log.enabled);
    }
}
