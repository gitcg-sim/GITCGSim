use super::*;

use crate::decl_support_impl_type;

pub const NAME: &str = "Liben";

pub const C: Card = Card {
    name: NAME,
    cost: Cost::ZERO,
    effects: list8![],
    card_type: CardType::Support(SupportType::Companion),
    card_impl: Some(&SupportImpl(SupportId::Liben)),
};

pub const S: Status = Status::new_indef(NAME, StatusAttachMode::Support).counter(CounterSpec::new("Liben", 0));

decl_support_impl_type!(Liben, I);
impl StatusImpl for Liben {
    fn responds_to(&self) -> EnumSet<RespondsTo> {
        enum_set![RespondsTo::TriggerEvent]
    }

    fn responds_to_triggers(&self) -> EnumSet<EventId> {
        enum_set![EventId::EndPhase]
    }

    fn trigger_event(&self, e: &mut TriggerEventContext) -> Option<AppliedEffectResult> {
        // TODO Each Omni can count as a different dice
        let EventId::EndPhase = e.event_id else { return None };
        let counter = e.c.eff_state.get_counter();
        if counter >= 3 {
            panic!("Liben: counter >= 3")
        }

        let Some((taken, collected)) = e.c.src_player_state.dice.try_collect(3 - counter) else {
            return Some(AppliedEffectResult::NoChange);
        };
        e.add_cmd(Command::SubtractDice(collected));
        let new_counter = counter + taken;
        if new_counter < 3 {
            Some(AppliedEffectResult::SetCounter(crate::std_subset::cmp::min(
                3,
                new_counter,
            )))
        } else {
            e.add_cmd(Command::DrawCards(2, None));
            e.add_cmd(Command::AddDice(DiceCounter::omni(2)));
            Some(AppliedEffectResult::DeleteSelf)
        }
    }
}
