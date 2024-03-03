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
        let player = &cic.players[cic.active_player_id];
        if player.char_states.iter_valid().any(|c| c.char_id == CharId::Keqing) {
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
        let player = &cic.players[cic.active_player_id];
        let Some((ci, _)) = player
            .char_states
            .enumerate_valid()
            .find(|(_, c)| c.char_id == CharId::Keqing)
        else {
            return;
        };
        let status_collection = cic.status_collections.get(cic.active_player_id);
        commands.push((*ctx, Command::SwitchCharacter(ci)));
        commands.append(&mut player.get_cast_skill_cmds(status_collection, ctx, SkillId::StellarRestoration));
    }
}
