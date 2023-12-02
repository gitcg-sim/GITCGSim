use super::*;

use crate::decl_support_impl_type;

pub const NAME: &str = "Iron Tongue Tian";

pub const C: Card = Card {
    name: NAME,
    cost: Cost::unaligned(2),
    effects: list8![],
    card_type: CardType::Support(SupportType::Companion),
    card_impl: Some(&SupportImpl(SupportId::IronTongueTian)),
};

pub const S: Status = Status::new_indef(NAME, StatusAttachMode::Support);

decl_support_impl_type!(IronTongueTian, I);
impl StatusImpl for IronTongueTian {
    fn responds_to(&self) -> EnumSet<RespondsTo> {
        enum_set![RespondsTo::TriggerEvent]
    }

    fn responds_to_triggers(&self) -> EnumSet<EventId> {
        enum_set![EventId::EndPhase]
    }

    fn trigger_event(&self, e: &mut TriggerEventContext) -> Option<AppliedEffectResult> {
        let EventId::EndPhase = e.event_id else { return None };
        let active_char = &e.c.src_player_state.char_states[e.active_char_idx()];
        if active_char.get_energy() < active_char.char_id.get_char_card().max_energy {
            e.add_cmd(Command::AddEnergy(1));
            return Some(AppliedEffectResult::ConsumeUsage);
        }

        let Some((char_idx, _)) =
            e.c.src_player_state
                .char_states
                .enumerate_valid()
                .find(|(_, c)| c.get_energy() < c.char_id.get_char_card().max_energy)
        else {
            return None;
        };
        // TODO
        e.add_cmd(Command::AddEnergyToCharacter(1, char_idx.into()));
        None
    }
}
