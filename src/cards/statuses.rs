use enumset::{enum_set, EnumSet};

use crate::decl_status_impl_type;
use crate::types::{card_defs::*, command::*, deal_dmg::*, enums::*, game_state::*, status_impl::*};

pub mod crystallize_shield {
    use super::*;
    pub const S: Status = Status::new_shield_points("Crystallize Shield", StatusAttachMode::Team, 1, Some(2));

    decl_status_impl_type!(CrystallizeShield, I);
    impl StatusImpl for CrystallizeShield {
        fn responds_to(&self) -> EnumSet<RespondsTo> {
            enum_set![]
        }
    }
}

pub mod dendro_core {
    use super::*;

    pub const S: Status = Status::new_usages("Dendro Core", StatusAttachMode::Team, 1, None);

    decl_status_impl_type!(DendroCore, I);
    impl StatusImpl for DendroCore {
        fn responds_to(&self) -> EnumSet<RespondsTo> {
            enum_set![RespondsTo::OutgoingDMG]
        }

        fn outgoing_dmg(&self, _: &StatusImplContext<DMGInfo>, dmg: &mut DealDMG) -> Option<AppliedEffectResult> {
            let DealDMGType::Elemental(Element::Pyro | Element::Electro) = dmg.dmg_type else {
                return None
            };
            dmg.dmg += 2;
            Some(AppliedEffectResult::ConsumeUsage)
        }
    }
}

pub mod catalyzing_field {
    use super::*;

    pub const S: Status = Status::new_usages("Catalyzing Field", StatusAttachMode::Team, 2, None);

    decl_status_impl_type!(CatalyzingField, I);
    impl StatusImpl for CatalyzingField {
        fn responds_to(&self) -> EnumSet<RespondsTo> {
            enum_set![RespondsTo::OutgoingDMG]
        }

        fn outgoing_dmg(&self, _: &StatusImplContext<DMGInfo>, dmg: &mut DealDMG) -> Option<AppliedEffectResult> {
            let DealDMGType::Elemental(Element::Dendro | Element::Electro) = dmg.dmg_type else {
                return None
            };
            dmg.dmg += 1;
            Some(AppliedEffectResult::ConsumeUsage)
        }
    }
}

pub mod frozen {
    use super::*;

    pub const S: Status = Status::new_indef("Frozen", StatusAttachMode::Character).with_applies_to_opposing();

    decl_status_impl_type!(Frozen, I);
    impl StatusImpl for Frozen {
        fn responds_to(&self) -> EnumSet<RespondsTo> {
            enum_set![RespondsTo::IncomingDMG | RespondsTo::CannotPerformActions | RespondsTo::TriggerEvent]
        }

        fn responds_to_triggers(&self) -> EnumSet<EventId> {
            enum_set![EventId::EndOfTurn]
        }

        fn incoming_dmg(&self, _: &StatusImplContext, dmg: &mut DealDMG) -> Option<AppliedEffectResult> {
            let (DealDMGType::Physical | DealDMGType::Elemental(Element::Pyro)) = dmg.dmg_type else {
                return None
            };
            dmg.dmg += 2;
            Some(AppliedEffectResult::DeleteSelf)
        }

        fn trigger_event(&self, _e: &mut TriggerEventContext) -> Option<AppliedEffectResult> {
            Some(AppliedEffectResult::DeleteSelf)
        }
    }
}

pub mod satiated {
    use super::*;

    pub const S: Status = Status::new_duration("Satiated", StatusAttachMode::Character, 1);

    pub const I: EmptyStatusImpl = EmptyStatusImpl();
}
