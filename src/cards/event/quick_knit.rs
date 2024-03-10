use super::*;

pub const C: Card = Card {
    name: "Quick Knit",
    cost: Cost::ONE,
    effects: list8![],
    card_type: CardType::Event,
    card_impl: Some(&I),
};

pub struct QuickKnit();

pub const I: QuickKnit = QuickKnit();

impl CardImpl for QuickKnit {
    fn can_be_played(&self, cic: &CardImplContext) -> CanBePlayedResult {
        let Some(CardSelection::OwnSummon(summon_id)) = cic.selection else {
            return CanBePlayedResult::InvalidSelection;
        };

        if summon_id.status().usages.is_some() {
            CanBePlayedResult::CanBePlayed
        } else {
            CanBePlayedResult::InvalidSelection
        }
    }

    fn selection(&self) -> Option<CardSelectionSpec> {
        Some(CardSelectionSpec::OwnSummon)
    }

    fn effects(
        &self,
        cic: &CardImplContext,
        ctx: &CommandContext,
        commands: &mut CommandList<(CommandContext, Command)>,
    ) {
        let Some(CardSelection::OwnSummon(summon_id)) = cic.selection else {
            panic!("Quick Knit: Invalid selection")
        };

        commands.push((*ctx, Command::IncreaseStatusUsages(StatusKey::Summon(summon_id), 1)))
    }
}
