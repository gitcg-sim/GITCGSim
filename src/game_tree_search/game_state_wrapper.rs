use rand::{
    rngs::{SmallRng, ThreadRng},
    seq::SliceRandom,
};
use serde::{Deserialize, Serialize};
use std::fmt::Debug;

use super::*;
use crate::{
    cards::ids::GetSkill,
    data_structures::ActionList,
    deck::Decklist,
    dispatcher_ops::types::DispatchError,
    rule_based::RuleBasedSearchConfig,
    tcg_model::enums::SkillType,
    types::{game_state::*, input::*, nondet::*},
};

#[derive(Clone, Serialize, Deserialize)]
pub struct GameStateWrapper<S: NondetState = StandardNondetHandlerState> {
    pub game_state: GameState,
    pub nd: NondetProvider<S>,
}

impl<S: NondetState> Debug for GameStateWrapper<S> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("GameStateWrapper")
            .field("game_state", &self.game_state)
            .finish()
    }
}

impl<S: NondetState> GameStateWrapper<S> {
    pub fn new(game_state: GameState, nd: NondetProvider<S>) -> Self {
        let mut new = Self { game_state, nd };
        new.ensure_player();
        new
    }

    pub fn ensure_player(&mut self) {
        while self.to_move().is_none() && self.winner().is_none() {
            let input = self.nd.get_no_to_move_player_input(&self.game_state);
            if let Err(e) = self.game_state.advance(input) {
                dbg!(&self.game_state);
                panic!("{e:?}\n{input:?}");
            }
        }
    }
}

pub fn new_standard_game(
    decklist1: &Decklist,
    decklist2: &Decklist,
    rng: SmallRng,
) -> GameStateWrapper<StandardNondetHandlerState> {
    let game_state = GameState::new(&decklist1.characters, &decklist2.characters, false);
    let state = StandardNondetHandlerState::new(decklist1, decklist2, rng.into());
    GameStateWrapper::new(game_state, NondetProvider::new(state))
}

impl<S: NondetState> ZobristHashable for GameStateWrapper<S> {
    #[inline]
    fn zobrist_hash(&self) -> u64 {
        self.game_state.zobrist_hash()
    }
}

impl<S: NondetState> Game for GameStateWrapper<S> {
    type Action = Input;

    type Actions = ActionList<Input>;

    type Eval = crate::minimax::Eval;

    type Error = DispatchError;

    const PREPARE_FOR_EVAL: bool = true;

    #[inline]
    fn winner(&self) -> Option<PlayerId> {
        match self.game_state.phase {
            Phase::WinnerDecided { winner } => Some(winner),
            _ => None,
        }
    }

    #[inline]
    fn to_move(&self) -> Option<PlayerId> {
        self.game_state.to_move_player()
    }

    #[inline]
    fn actions(&self) -> Self::Actions {
        self.game_state.available_actions()
    }

    #[inline]
    fn hide_private_information(&mut self, player_to_hide: PlayerId) {
        self.game_state.log.enabled = false;
        self.nd.hide_private_information(&mut self.game_state, player_to_hide);
    }

    fn convert_to_tactical_search(&mut self) {
        self.game_state.convert_to_tactical_search();
        self.nd
            .hide_private_information(&mut self.game_state, PlayerId::PlayerFirst);
        self.nd
            .hide_private_information(&mut self.game_state, PlayerId::PlayerSecond);
    }

    #[inline]
    fn advance(&mut self, action: Input) -> Result<(), Self::Error> {
        let _ = self.game_state.advance(action)?;
        self.ensure_player();
        Ok(())
    }

    #[inline]
    fn prepare_for_eval(&mut self) {
        const ROUNDS: u8 = 2;
        fn try_skip_round(game_state: &mut GameState) -> bool {
            while game_state.phase.winner().is_some() {
                let actions = game_state.available_actions();
                if actions.is_empty() {
                    return false;
                }
                if let Some(end_round) = actions
                    .iter()
                    .copied()
                    .find(|a| matches!(a, Input::FromPlayer(.., PlayerAction::EndRound)))
                {
                    game_state.advance(end_round).expect("Can't perform end round");
                    return true;
                } else {
                    game_state.advance(actions[0]).expect("Can't perform skipping action");
                }
            }
            false
        }

        for _ in 0..ROUNDS {
            if self.winner().is_some() {
                break;
            }
            if !try_skip_round(&mut self.game_state) {
                break;
            }
            if let Phase::ActionPhase {
                first_end_round: Some(..),
                ..
            } = self.game_state.phase
            {
                if !try_skip_round(&mut self.game_state) {
                    break;
                }
            }
        }
    }

    #[inline]
    fn eval(&self, player_id: PlayerId) -> Self::Eval {
        let e1 = self.game_state.get_player(player_id).eval();
        let e2 = self.game_state.get_player(player_id.opposite()).eval();
        Self::Eval::from_heuristic(e1 - e2)
    }

    #[inline]
    fn round_number(&self) -> u8 {
        self.game_state.round_number
    }

    fn shuffle_actions(actions: &mut Self::Actions, rng: &mut ThreadRng) {
        actions.shuffle(rng);
    }

    fn move_ordering(&self, pv: &PV<Self>, actions: &mut Self::Actions) {
        let Some(player_id) = self.to_move() else {
            return
        };
        let next_tactical_action = pv.clone().find(|a| Self::is_tactical_action(*a));
        let scores = RuleBasedSearchConfig::DEFAULT.action_scores(self, actions, player_id);
        actions[1..].sort_by_key(|&action| {
            let score = scores
                .iter()
                .find_map(|(a, s)| if *a == action { Some(*s) } else { None })
                .unwrap_or(0);
            if next_tactical_action == Some(action) {
                210u8
            } else {
                200 - score
            }
        });
    }

    fn static_search_action(&self, player_id: PlayerId) -> Option<Self::Action> {
        let actions = self.actions();
        let use_scores = false;
        if use_scores {
            let mut scores = RuleBasedSearchConfig::DEFAULT.action_scores(self, &actions, player_id);
            scores.sort_by_key(|(_, v)| *v);
            scores.last().map(|(a, _)| *a)
        } else {
            actions.first().copied()
        }
    }

    #[inline]
    fn is_tactical_action(action: Self::Action) -> bool {
        matches!(
            action,
            Input::FromPlayer(
                _,
                PlayerAction::SwitchCharacter(..) | PlayerAction::CastSkill(..) | PlayerAction::EndRound
            )
        )
    }

    fn move_ordering_for_qs(actions: &mut Self::Actions) {
        actions.sort_by_key(|a| match *a {
            Input::FromPlayer(_, PlayerAction::CastSkill(skill_id)) => match skill_id.get_skill().skill_type {
                SkillType::ElementalBurst => 1u8,
                SkillType::ElementalSkill => 2,
                SkillType::NormalAttack => 3,
            },
            Input::FromPlayer(_, PlayerAction::SwitchCharacter(..)) => 4,
            Input::FromPlayer(_, PlayerAction::EndRound) => 5,
            _ => 3,
        })
    }
}
