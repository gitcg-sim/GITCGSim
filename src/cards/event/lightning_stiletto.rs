use crate::{data_structures::CommandList, dispatcher_ops::get_cast_skill_cmds};

use super::*;

pub const C: Card = Card {
    name: "Lightning Stiletto",
    cost: Cost::elem(Element::Electro, 3),
    effects: list8![],
    card_type: CardType::Event,
    card_impl: Some(&I),
};

pub struct LightningStiletto();

pub const I: LightningStiletto = LightningStiletto();

impl CardImpl for LightningStiletto {
    fn can_be_played(&self, cic: &CardImplContext) -> CanBePlayedResult {
        let player = cic.game_state.get_player(cic.active_player_id);
        if player
            .char_states
            .iter()
            .enumerate()
            .any(|(ci, c)| player.is_valid_char_index(ci as u8) && c.char_id == CharId::Keqing)
        {
            CanBePlayedResult::CanBePlayed
        } else {
            CanBePlayedResult::CannotBePlayed
        }
    }

    fn get_effects(
        &self,
        cic: &CardImplContext,
        ctx: &CommandContext,
        commands: &mut CommandList<(CommandContext, Command)>,
    ) {
        let player = cic.game_state.get_player(cic.active_player_id);
        let Some((ci, _)) = player.char_states.iter().enumerate()
            .find(|(ci, c)| player.is_valid_char_index(*ci as u8) && c.char_id == CharId::Keqing) else {
            return
        };
        commands.push((*ctx, Command::SwitchCharacter(ci as u8)));
        commands.append(&mut get_cast_skill_cmds(player, ctx, SkillId::StellarRestoration));
    }
}
