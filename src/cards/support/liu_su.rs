use super::*;

use crate::decl_support_impl_type;

pub const NAME: &str = "Liu Su";

pub const C: Card = Card {
    name: NAME,
    cost: Cost::ONE,
    effects: list8![],
    card_type: CardType::Support(SupportType::Companion),
    card_impl: Some(&SupportImpl(SupportId::LiuSu)),
};

pub const S: Status = Status::new_usages(NAME, StatusAttachMode::Support, 2, None);

// TODO implement once per round per name (not per instance)
decl_support_impl_type!(LiuSu, I);
impl StatusImpl for LiuSu {
    fn responds_to(&self) -> EnumSet<RespondsTo> {
        enum_set![RespondsTo::TriggerEvent]
    }

    fn responds_to_triggers(&self) -> EnumSet<EventId> {
        enum_set![EventId::Switched]
    }

    fn trigger_event(&self, e: &mut TriggerEventContext) -> Option<AppliedEffectResult> {
        let EventId::Switched = e.event_id else { return None };
        let Some(dst_char_idx) = e.c.ctx.src.switch_dst_char_idx() else {
            return None;
        };
        let char_state = &e.c.src_player_state.char_states[dst_char_idx];
        if char_state.energy() < char_state.char_id.char_card().max_energy {
            e.add_cmd(Command::AddEnergy(1, CmdCharIdx::Active));
            Some(AppliedEffectResult::ConsumeUsage)
        } else {
            None
        }
    }
}
