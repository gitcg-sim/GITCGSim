use crate::types::by_player::ByPlayer;

use super::*;

impl GameState {
    pub(crate) fn nondet_result_to_commands(
        &self,
        res: NondetResult,
        cmds: &mut CommandList<(CommandContext, Command)>,
    ) {
        let ctx1 = CommandContext::new(PlayerId::PlayerFirst, CommandSource::Event, None);
        let ctx2 = CommandContext::new(PlayerId::PlayerSecond, CommandSource::Event, None);
        match res {
            NondetResult::ProvideDice(ByPlayer(d1, d2)) => {
                if !d1.is_empty() {
                    cmds.push((ctx1, Command::AddDice(d1)))
                }
                if !d2.is_empty() {
                    cmds.push((ctx2, Command::AddDice(d2)))
                }
            }
            NondetResult::ProvideCards(ByPlayer(c1, c2)) => {
                if !c1.is_empty() {
                    cmds.push((ctx1, Command::AddCardsToHand(c1)))
                }

                if !c2.is_empty() {
                    cmds.push((ctx2, Command::AddCardsToHand(c2)))
                }
            }
            NondetResult::ProvideSummonIds(summon_ids) => {
                for &summon_id in summon_ids.iter() {
                    cmds.push((ctx1, Command::Summon(summon_id)))
                }
            }
        }
    }
}
