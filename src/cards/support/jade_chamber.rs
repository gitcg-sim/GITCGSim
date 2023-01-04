use super::*;

pub const NAME: &str = "Jade Chamber";

pub const C: Card = Card {
    name: NAME,
    cost: Cost::ONE,
    effects: list8![Command::RerollDice],
    card_type: CardType::Support(SupportType::Location),
    card_impl: Some(&SupportImpl(SupportId::JadeChamber)),
};

pub const S: Status = Status::new_indef(NAME, StatusAttachMode::Support);

decl_support_impl_type!(JadeChamber, I);
impl StatusImpl for JadeChamber {
    fn responds_to(&self) -> EnumSet<RespondsTo> {
        enum_set![RespondsTo::DiceDistribution]
    }

    fn dice_distribution(&self, c: &StatusImplContext, dist: &mut DiceDistribution) -> bool {
        let active_char_elem = c.src_player_state.char_states[c.src_player_state.active_char_index as usize]
            .char_id
            .get_char_card()
            .elem;
        dist.guarantee_elem(active_char_elem, 2)
    }
}
