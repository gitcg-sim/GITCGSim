use super::*;

pub const C: Card = Card {
    name: "Send Off",
    cost: Cost::unaligned(2),
    effects: list8![],
    card_type: CardType::Event,
    card_impl: Some(&I),
};

pub struct SendOff();

pub const I: SendOff = SendOff();

impl CardImpl for SendOff {
    fn can_be_played(&self, cic: &CardImplContext) -> CanBePlayedResult {
        let Some(CardSelection::OpponentSummon(..)) = cic.selection else {
            return CanBePlayedResult::InvalidSelection
        };

        CanBePlayedResult::CanBePlayed
    }

    fn selection(&self) -> Option<CardSelectionSpec> {
        Some(CardSelectionSpec::OpponentSummon)
    }

    fn get_effects(
        &self,
        cic: &CardImplContext,
        ctx: &CommandContext,
        commands: &mut CommandList<(CommandContext, Command)>,
    ) {
        let Some(CardSelection::OpponentSummon(summon_id)) = cic.selection else {
            panic!("Send Off: Invalid selection")
        };

        commands.push((*ctx, Command::DeleteStatusForTarget(StatusKey::Summon(summon_id))))
    }
}
