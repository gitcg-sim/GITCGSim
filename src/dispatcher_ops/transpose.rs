use super::types::NondetRequest;
use crate::status_impls::prelude::{Command, XEvent};
use crate::types::by_player::ByPlayer;
use crate::types::command::CommandContext;
use crate::types::game_state::*;
use crate::types::input::{Input, NondetResult};

impl Input {
    /// Swaps the roles of the two players.
    /// See also: `GameState::transpose_in_place`
    pub fn transpose_in_place(&mut self) {
        match self {
            Input::NoAction => {}
            Input::NondetResult(r) => r.transpose_in_place(),
            Input::FromPlayer(p, _) => p.flip(),
        }
    }

    /// Swaps the roles of the two players.
    /// See also: `GameState::transpose`
    pub fn transpose(self) -> Self {
        let mut input = self;
        input.transpose_in_place();
        input
    }
}

impl XEvent {
    fn transpose_in_place(&mut self) {
        match self {
            XEvent::DMG(dmg) => dmg.src_player_id.flip(),
            XEvent::Skill(skill) => skill.src_player_id.flip(),
        }
    }
}

impl Command {
    fn transpose_in_place(&mut self) {
        if let Command::TriggerXEvent(xevt) = self {
            xevt.transpose_in_place()
        }
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
            NondetResult::ProvideDice(ByPlayer(a, b)) => crate::std_subset::mem::swap(a, b),
            NondetResult::ProvideCards(ByPlayer(a, b)) => crate::std_subset::mem::swap(a, b),
            NondetResult::ProvideSummonIds(..) => {}
        }
    }
}

impl PlayerId {
    #[inline(always)]
    fn flip(&mut self) {
        *self = self.opposite();
    }
}

impl SelectStartingCharacterState {
    #[inline(always)]
    fn flip(&mut self) {
        match self {
            SelectStartingCharacterState::Start { to_select } => to_select.flip(),
            SelectStartingCharacterState::FirstSelected { to_select } => to_select.flip(),
        }
    }
}

impl ByPlayer<PlayerState> {
    #[inline(always)]
    fn transpose_in_place(&mut self) {
        // Nothing to transpose for PlayerState
        crate::std_subset::mem::swap(&mut self.0, &mut self.1);
    }
}

impl Phase {
    pub fn transpose_in_place(&mut self) {
        match self {
            Phase::SelectStartingCharacter { state } => state.flip(),
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
            NondetRequest::DrawCards(ByPlayer(a, b)) => crate::std_subset::mem::swap(a, b),
            NondetRequest::DrawCardsOfType(p, _, _) => p.flip(),
            NondetRequest::RollDice(ByPlayer(a, b)) => crate::std_subset::mem::swap(a, b),
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
        for (ctx, cmd) in self.pending_cmds.iter_mut() {
            ctx.transpose_in_place();
            cmd.transpose_in_place();
        }
    }
}

impl GameState {
    /// Swaps the roles of the two players.
    /// See also: `Input::transpose_in_place`
    pub fn transpose_in_place(&mut self) {
        self.players.transpose_in_place();
        if let Some(pending_cmds) = &mut self.pending_cmds {
            pending_cmds.transpose_in_place();
        }
        self.phase.transpose_in_place();
        self.rehash();
    }

    /// Swaps the roles of the two players.
    /// See also: `Input::transpose`
    pub fn transpose(&self) -> Self {
        let mut gs = self.clone();
        gs.transpose_in_place();
        gs
    }
}
