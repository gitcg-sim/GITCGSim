use super::*;

pub struct FoodCardImpl();
impl CardImpl for FoodCardImpl {
    fn selection(&self) -> Option<CardSelectionSpec> {
        Some(CardSelectionSpec::OwnCharacter)
    }

    fn can_be_played(&self, cic: &CardImplContext) -> CanBePlayedResult {
        let Some(CardSelection::OwnCharacter(char_idx)) = cic.selection else {
            return CanBePlayedResult::InvalidSelection;
        };
        let status_collection = &cic.status_collections[cic.active_player_id];
        if status_collection.has_character_status(char_idx, StatusId::Satiated) {
            CanBePlayedResult::InvalidSelection
        } else {
            CanBePlayedResult::CanBePlayed
        }
    }

    fn effects(
        &self,
        cic: &CardImplContext,
        ctx: &CommandContext,
        commands: &mut crate::data_structures::CommandList<(CommandContext, Command)>,
    ) {
        for &eff in cic.card.effects.iter() {
            commands.push((*ctx, eff))
        }
        if let CardSelection::OwnCharacter(i) = cic.selection.expect("FoodCardImpl: must have selection") {
            commands.push((*ctx, Command::ApplyCharacterStatus(StatusId::Satiated, i.into())));
        }
    }
}

pub mod sweet_madame {
    use super::*;

    pub const C: Card = Card {
        name: "Sweet Madame",
        cost: Cost::ZERO,
        effects: list8![Command::Heal(1, CmdCharIdx::CardSelected)],
        card_type: CardType::Food,
        card_impl: Some(&FoodCardImpl()),
    };
}

pub mod mondstadt_hash_brown {
    use super::*;

    pub const C: Card = Card {
        name: "Mondstadt Hash Brown",
        cost: Cost::ONE,
        effects: list8![Command::Heal(2, CmdCharIdx::CardSelected)],
        card_type: CardType::Food,
        card_impl: Some(&FoodCardImpl()),
    };
}

pub mod mushroom_pizza;

pub mod lotus_flower_crisp;

pub mod adeptus_temptation;

pub mod minty_meat_rolls;

pub mod northern_smoked_chicken;
