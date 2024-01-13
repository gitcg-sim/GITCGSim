use super::*;

use gitcg_sim::{
    data_structures::ActionList,
    prelude::*,
    rand::{rngs::ThreadRng, seq::SliceRandom},
    rule_based::RuleBasedSearchConfig,
    smallvec::SmallVec,
};

impl ValueTrait for Input {}
impl<S: NondetState> Game for GameStateWrapper<S> {
    type Action = Input;

    type Actions = ActionList<Input>;

    type Eval = crate::minimax::Eval;

    type Error = DispatchError;

    const PREPARE_FOR_EVAL: bool = true;

    #[inline]
    fn winner(&self) -> Option<PlayerId> {
        Self::winner(self)
    }

    #[inline]
    fn to_move(&self) -> Option<PlayerId> {
        Self::to_move(self)
    }

    #[inline]
    fn actions(&self) -> Self::Actions {
        Self::actions(self)
    }

    #[inline]
    fn hide_private_information(&mut self, player_to_hide: PlayerId) {
        Self::hide_private_information(self, player_to_hide)
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
        let h = e1 - e2;
        if let Some(winner) = self.winner() {
            if winner == player_id {
                Self::Eval::win()
            } else {
                Self::Eval::lose()
            }
        } else {
            Self::Eval::from_eval(h)
        }
    }

    #[inline]
    fn round_number(&self) -> u8 {
        self.game_state.round_number
    }

    fn shuffle_actions(actions: &mut Self::Actions, rng: &mut ThreadRng) {
        actions.shuffle(rng);
    }

    fn move_ordering(&self, pv: &PV<Self>, actions: &mut Self::Actions) {
        let Some(player_id) = self.to_move() else { return };
        const LOOKAHEAD: usize = 4;
        let move_chain = pv
            .into_iter()
            .take(LOOKAHEAD)
            .filter(|a| a.player() == Some(player_id))
            .collect::<SmallVec<[_; LOOKAHEAD]>>();

        let scores = RuleBasedSearchConfig::DEFAULT.action_scores(self, actions, player_id);
        actions.sort_by_key(|&action| {
            let index_from_move_chain = move_chain
                .iter()
                .copied()
                .copied()
                .enumerate()
                .find(|(_, a)| action == *a)
                .map(|x| x.0);
            // score >= 0
            let score = scores
                .iter()
                .find_map(|(a, s)| if *a == action { Some(*s) } else { None })
                .unwrap_or(0) as i16;
            if index_from_move_chain == Some(0) {
                -1100
            } else if index_from_move_chain == Some(move_chain.len() - 1) {
                -1080
            } else if index_from_move_chain.is_some() {
                -1060
            } else {
                -score
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

    fn action_weights(&self, actions: &Self::Actions) -> Vec<(Self::Action, f32)> {
        let Some(player_id) = self.to_move() else {
            return Default::default();
        };
        let scores = RuleBasedSearchConfig::DEFAULT
            .action_scores(self, actions, player_id)
            .iter()
            .copied()
            .map(|x| (x.0, 1e-6 + (x.1 as f32)))
            .collect::<Vec<_>>();
        let tot = scores.iter().map(|(_, x)| *x).sum::<f32>().clamp(1.0, 1e6);
        scores.iter().map(|x| (x.0, x.1 / tot)).collect()
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

    #[inline]
    fn depth_extension(&self, action: Self::Action) -> u8 {
        let Some(player_id) = self.to_move() else { return 0 };
        let player = self.game_state.get_player(player_id);
        if !player.is_tactical() {
            return 0;
        }

        if matches!(
            action,
            Input::FromPlayer(
                ..,
                PlayerAction::ElementalTuning(..) | PlayerAction::SwitchCharacter(..)
            )
        ) {
            1
        } else {
            0
        }
    }
}
