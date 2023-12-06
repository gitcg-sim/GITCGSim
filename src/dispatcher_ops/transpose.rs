use super::types::NondetRequest;
use crate::types::by_player::ByPlayer;
use crate::types::command::CommandContext;
use crate::types::game_state::*;
use crate::types::input::{Input, NondetResult};

impl Input {
    pub fn transpose_in_place(&mut self) {
        match self {
            Input::NoAction => {},
            Input::NondetResult(r) => r.transpose_in_place(),
            Input::FromPlayer(p, c) => p.flip(),
        }
    }

    pub fn transpose(self) -> Self {
        let mut x = self;
        x.transpose_in_place();
        x
    }
}

impl CommandContext {
    fn transpose_in_place(&mut self) {
        self.src_player_id.flip();
        if let Some(tgt) = &mut self.tgt {
            tgt.player_id.flip();
        }
    }
}

impl NondetResult {
    fn transpose_in_place(&mut self) {
        match self {
            NondetResult::ProvideDice(a, b) => std::mem::swap(a, b),
            NondetResult::ProvideCards(a, b) => std::mem::swap(a, b),
            NondetResult::ProvideSummonIds(x) => {},
        }
    }
}

impl PlayerId {
    #[inline(always)]
    fn flip(&mut self) {
        *self = self.opposite();
    }
}

impl ByPlayer<PlayerState> {
    #[inline(always)]
    fn transpose_in_place(&mut self) {
        // Nothing to transpose for PlayerState
        std::mem::swap(&mut self.0, &mut self.1);
    }
}

impl Phase {
    pub fn transpose_in_place(&mut self) {
        match self {
            Phase::SelectStartingCharacter {
                already_selected: Some(player_id),
            } => {
                player_id.flip();
            }
            Phase::SelectStartingCharacter { already_selected: None } => {}
            Phase::RollPhase {
                first_active_player,
                roll_phase_state: _,
            } => {
                first_active_player.flip();
            }
            Phase::ActionPhase {
                first_end_round,
                active_player,
            } => {
                if let Some(first_end_round) = first_end_round {
                    first_end_round.flip();
                }
                active_player.flip();
            }
            Phase::EndPhase {
                next_first_active_player,
            } => {
                next_first_active_player.flip();
            }
            Phase::WinnerDecided { winner } => {
                winner.flip();
            }
        }
    }
}

impl NondetRequest {
    pub fn transpose_in_place(&mut self) {
        match self {
            NondetRequest::DrawCards(a, b) => std::mem::swap(a, b),
            NondetRequest::DrawCardsOfType(p, _, _) => p.flip(),
            NondetRequest::RollDice(a, b) => std::mem::swap(a, b),
            NondetRequest::SummonRandom(..) => {}
        }
    }
}

impl PendingCommands {
    pub fn transpose_in_place(&mut self) {
        match &mut self.suspended_state {
            SuspendedState::PostDeathSwitch {
                player_id,
                character_statuses_to_shift: _,
            } => player_id.flip(),
            SuspendedState::NondetRequest(req) => req.transpose_in_place(),
        }
        for (ctx, _) in self.pending_cmds.iter_mut() {
            ctx.transpose_in_place();
        }
    }
}

impl GameState {
    /// Swaps the roles of the two players.
    /// See also: `Input::transpose`
    pub fn transpose_in_place(&mut self) {
        self.players.transpose_in_place();
        if let Some(pending_cmds) = &mut self.pending_cmds {
            pending_cmds.transpose_in_place();
        }
        self.phase.transpose_in_place();
        self.rehash();
    }
}
