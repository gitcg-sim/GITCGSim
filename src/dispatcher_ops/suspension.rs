use super::types::*;
use crate::{
    action_list, cmd_list,
    data_structures::{ActionList, CommandList},
    phc,
    types::{command::*, game_state::*, input::*},
};

impl SuspendedState {
    #[inline]
    pub fn to_move_player(&self) -> Option<PlayerId> {
        match self {
            SuspendedState::PostDeathSwitch { player_id, .. } => Some(*player_id),
            SuspendedState::NondetRequest(..) => None,
        }
    }

    #[inline]
    pub fn get_nondet_request(&self) -> Option<NondetRequest> {
        match self {
            SuspendedState::PostDeathSwitch { .. } => None,
            SuspendedState::NondetRequest(req) => Some(*req),
        }
    }

    #[inline]
    pub fn get_dispatch_result(&self) -> DispatchResult {
        match self {
            SuspendedState::PostDeathSwitch { player_id, .. } => DispatchResult::PlayerInput(*player_id),
            SuspendedState::NondetRequest(req) => DispatchResult::NondetRequest(*req),
        }
    }

    pub fn available_actions(&self, game_state: &GameState) -> ActionList<Input> {
        let mut acts = action_list![];
        match *self {
            SuspendedState::PostDeathSwitch { player_id, .. } => {
                let p = game_state.get_player(player_id);
                for char_idx in 0..p.char_states.len() {
                    if p.is_valid_char_index(char_idx as u8) {
                        acts.push(Input::FromPlayer(
                            player_id,
                            PlayerAction::PostDeathSwitch(char_idx as u8),
                        ));
                    }
                }
                acts
            }
            SuspendedState::NondetRequest(..) => acts,
        }
    }
}

impl GameState {
    fn resolve_post_death_switch(
        &mut self,
        input: Input,
        player_id: PlayerId,
    ) -> Result<(CommandContext, Command), DispatchError> {
        match input {
            Input::NondetResult(..) => Err(DispatchError::NondetResultNotAllowed),
            Input::NoAction => Err(DispatchError::InvalidPlayer),
            Input::FromPlayer(p, _) if p != player_id => Err(DispatchError::InvalidPlayer),
            Input::FromPlayer(_, action) => match action {
                PlayerAction::PostDeathSwitch(char_index) => {
                    let player = self.players.get_mut(player_id);
                    if !player.is_valid_char_index(char_index) {
                        Err(DispatchError::CannotSwitchInto)
                    } else {
                        let prev_char_idx = player.active_char_index;
                        player.update_active_char_index(phc!(self, player_id), char_index);
                        Ok(Self::trigger_switch_cmd(player_id, prev_char_idx, char_index))
                    }
                }
                PlayerAction::SwitchCharacter(_) => Err(DispatchError::InvalidInput(
                    "post_death_switch: Use PostDeathSwitch instead.".to_string(),
                )),
                _ => Err(DispatchError::InvalidInput(
                    "post_death_switch: Invalid input.".to_string(),
                )),
            },
        }
    }

    fn resolve_nondet_request(
        &mut self,
        input: Input,
        req: NondetRequest,
        cmds: &mut CommandList<(CommandContext, Command)>,
    ) -> Result<(), DispatchError> {
        match input {
            Input::NoAction | Input::FromPlayer(..) => Err(DispatchError::InvalidPlayer),
            Input::NondetResult(res) => {
                let correct = match (req, res) {
                    (NondetRequest::DrawCards(..), NondetResult::ProvideCards(..)) => true,
                    (NondetRequest::DrawCardsOfType(..), NondetResult::ProvideCards(..)) => true,
                    (NondetRequest::RollDice(..), NondetResult::ProvideDice(..)) => true,
                    (NondetRequest::SummonRandom(..), NondetResult::ProvideSummonIds(..)) => true,
                    (_, _) => false,
                };
                if !correct {
                    return Err(DispatchError::NondetResultInvalid);
                }
                self.nondet_result_to_commands(res, cmds);
                Ok(())
            }
        }
    }

    pub(crate) fn resolve_pending_cmds(&mut self, input: Input) -> Result<Option<DispatchResult>, DispatchError> {
        let pc = self.pending_cmds.take().unwrap();
        let mut c = cmd_list![];
        match pc.suspended_state {
            SuspendedState::PostDeathSwitch { player_id, .. } => {
                // TODO handle statuses to shift
                c.push(self.resolve_post_death_switch(input, player_id)?);
            }
            SuspendedState::NondetRequest(req) => self.resolve_nondet_request(input, req, &mut c)?,
        };
        self.pending_cmds = None;
        for pc1 in pc.pending_cmds {
            c.push(pc1);
        }
        self.exec_commands(&c)
    }
}
