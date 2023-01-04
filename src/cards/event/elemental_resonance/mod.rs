use super::*;
use crate::reaction::*;

macro_rules! elemental_resonance_0 {
    ($elem: expr, $name: expr) => {
        use super::*;
        const NAME: &str = $name;

        pub const C: Card = Card {
            name: NAME,
            cost: Cost::ZERO,
            effects: list8![Command::AddDice(DiceCounter::elem($elem, 1))],
            card_type: CardType::ElementalResonance($elem),
            card_impl: None,
        };
    };
}

pub mod elemental_resonance_woven_flames {
    elemental_resonance_0!(Element::Pyro, "Elemental Resonance: Woven Flames");
}

pub mod elemental_resonance_woven_ice {
    elemental_resonance_0!(Element::Cryo, "Elemental Resonance: Woven Ice");
}

pub mod elemental_resonance_woven_stone {
    elemental_resonance_0!(Element::Geo, "Elemental Resonance: Woven Stone");
}

pub mod elemental_resonance_woven_thunder {
    elemental_resonance_0!(Element::Electro, "Elemental Resonance: Woven Thunder");
}

pub mod elemental_resonance_woven_waters {
    elemental_resonance_0!(Element::Hydro, "Elemental Resonance: Woven Waters");
}

pub mod elemental_resonance_woven_weeds {
    elemental_resonance_0!(Element::Dendro, "Elemental Resonance: Woven Weeds");
}

pub mod elemental_resonance_woven_winds {
    elemental_resonance_0!(Element::Anemo, "Elemental Resonance: Woven Winds");
}

pub mod elemental_resonance_shattering_ice {
    use super::*;

    const NAME: &str = "Elemental Resonance: Shattering Ice";

    pub const C: Card = Card {
        name: NAME,
        cost: Cost::elem(Element::Cryo, 1),
        effects: list8![Command::ApplyStatusToTeam(StatusId::ElementalResonanceShatteringIce)],
        card_type: CardType::ElementalResonance(Element::Cryo),
        card_impl: None,
    };

    pub const S: Status = Status::new_duration(NAME, StatusAttachMode::Team, 1);

    decl_status_impl_type!(ElementalResonanceShatteringIce, I);
    impl StatusImpl for ElementalResonanceShatteringIce {
        fn responds_to(&self) -> EnumSet<RespondsTo> {
            enum_set![RespondsTo::OutgoingDMG]
        }

        fn outgoing_dmg(&self, _: &StatusImplContext<DMGInfo>, dmg: &mut DealDMG) -> Option<AppliedEffectResult> {
            dmg.dmg += 2;
            Some(AppliedEffectResult::DeleteSelf)
        }
    }
}

pub mod elemental_resonance_fervent_flames {
    use super::*;

    const NAME: &str = "Elemental Resonance: Fervent Flames";

    pub const C: Card = Card {
        name: NAME,
        cost: Cost::elem(Element::Pyro, 1),
        effects: list8![Command::ApplyStatusToTeam(StatusId::ElementalResonanceFerventFlames)],
        card_type: CardType::ElementalResonance(Element::Pyro),
        card_impl: None,
    };

    pub const S: Status = Status::new_duration(NAME, StatusAttachMode::Team, 1);

    decl_status_impl_type!(ElementalResonanceFerventFlames, I);
    impl StatusImpl for ElementalResonanceFerventFlames {
        fn responds_to(&self) -> EnumSet<RespondsTo> {
            enum_set![RespondsTo::OutgoingReactionDMG]
        }

        fn outgoing_reaction_dmg(
            &self,
            _: &StatusImplContext<DMGInfo>,
            (reaction, _): (Reaction, Option<Element>),
            dmg: &mut DealDMG,
        ) -> Option<AppliedEffectResult> {
            if !PYRO_REACTIONS.contains(reaction) {
                return None;
            }

            dmg.dmg += 3;
            Some(AppliedEffectResult::DeleteSelf)
        }
    }
}

pub mod elemental_resonance_high_voltage {
    use super::*;

    const NAME: &str = "Elemental Resonance: High Voltage";

    pub const C: Card = Card {
        name: NAME,
        cost: Cost::elem(Element::Electro, 1),
        effects: list8![Command::AddEnergyWithoutMaximum(1)],
        card_type: CardType::ElementalResonance(Element::Electro),
        card_impl: None,
    };
}

pub mod elemental_resonance_sprawling_greenery {
    use super::*;

    const NAME: &str = "Elemental Resonance: Sprawling Greenery";

    pub const C: Card = Card {
        name: NAME,
        cost: Cost::elem(Element::Dendro, 1),
        effects: list8![
            Command::ApplyStatusToTeam(StatusId::ElementalResonanceSprawlingGreenery),
            Command::IncreaseStatusUsages(StatusKey::Team(StatusId::DendroCore), 1),
            Command::IncreaseStatusUsages(StatusKey::Team(StatusId::CatalyzingField), 1),
            Command::IncreaseStatusUsages(StatusKey::Summon(SummonId::BurningFlame), 1),
        ],
        card_type: CardType::ElementalResonance(Element::Dendro),
        card_impl: None,
    };

    pub const S: Status = Status::new_duration(NAME, StatusAttachMode::Team, 1);

    decl_status_impl_type!(ElementalResonanceSprawlingGreenery, I);
    impl StatusImpl for ElementalResonanceSprawlingGreenery {
        fn responds_to(&self) -> EnumSet<RespondsTo> {
            enum_set![RespondsTo::OutgoingReactionDMG]
        }

        fn outgoing_reaction_dmg(
            &self,
            c: &StatusImplContext<DMGInfo>,
            _: (Reaction, Option<Element>),
            dmg: &mut DealDMG,
        ) -> Option<AppliedEffectResult> {
            if !c.eff_state.can_use_once_per_round() {
                return None;
            }
            dmg.dmg += 2;
            Some(AppliedEffectResult::ConsumeOncePerRound)
        }
    }
}
