use enumset::{enum_set, EnumSet};

use crate::status_impls::prelude::RelativeCharIdx;
use crate::types::ElementSet;
use crate::{
    cards::ids::*,
    types::{command::Command, enums::*},
};

macro_rules! any_order {
    ($a: pat, $b: pat) => {
        ($a, $b) | ($b, $a)
    };
}

pub fn check_reaction(elem1: Element, elem2: Element) -> (Option<Reaction>, Option<Element>) {
    match (elem1, elem2) {
        any_order!(Element::Dendro, Element::Hydro) => (Some(Reaction::Bloom), None),
        any_order!(Element::Dendro, Element::Pyro) => (Some(Reaction::Burning), None),
        any_order!(Element::Geo, e) if e.is_phec() => (Some(Reaction::Crystallize), Some(e)),
        any_order!(Element::Electro, Element::Hydro) => (Some(Reaction::ElectroCharged), None),
        any_order!(Element::Cryo, Element::Hydro) => (Some(Reaction::Frozen), None),
        any_order!(Element::Cryo, Element::Pyro) => (Some(Reaction::Melt), None),
        any_order!(Element::Electro, Element::Pyro) => (Some(Reaction::Overloaded), None),
        any_order!(Element::Dendro, Element::Electro) => (Some(Reaction::Quicken), None),
        any_order!(Element::Cryo, Element::Electro) => (Some(Reaction::Superconduct), None),
        any_order!(Element::Anemo, e) => (Some(Reaction::Swirl), Some(e)),
        any_order!(Element::Hydro, Element::Pyro) => (Some(Reaction::Vaporize), None),
        _ => (None, None),
    }
}

pub const PYRO_REACTIONS: EnumSet<Reaction> =
    enum_set![Reaction::Burning | Reaction::Melt | Reaction::Overloaded | Reaction::Crystallize];

pub const HYDRO_REACTIONS: EnumSet<Reaction> =
    enum_set![Reaction::Bloom | Reaction::ElectroCharged | Reaction::Frozen | Reaction::Vaporize];

pub const DENDRO_REACTIONS: EnumSet<Reaction> = enum_set![Reaction::Bloom | Reaction::Burning | Reaction::Quicken];

pub const SWIRL_PYRO: Option<Command> = Some(Command::InternalDealSwirlDMG(Element::Pyro, 1));
pub const SWIRL_HYDRO: Option<Command> = Some(Command::InternalDealSwirlDMG(Element::Hydro, 1));
pub const SWIRL_ELECTRO: Option<Command> = Some(Command::InternalDealSwirlDMG(Element::Electro, 1));
pub const SWIRL_CRYO: Option<Command> = Some(Command::InternalDealSwirlDMG(Element::Cryo, 1));

impl Element {
    pub fn is_phec(&self) -> bool {
        matches!(self, Element::Cryo | Element::Electro | Element::Hydro | Element::Pyro)
    }

    pub fn can_be_applied(&self) -> bool {
        !matches!(self, Element::Geo | Element::Anemo)
    }

    fn swirl_effect(&self) -> Option<Command> {
        match self {
            Element::Pyro => SWIRL_PYRO,
            Element::Hydro => SWIRL_HYDRO,
            Element::Electro => SWIRL_ELECTRO,
            Element::Cryo => SWIRL_CRYO,
            _ => None,
        }
    }

    #[inline]
    pub fn to_index(&self) -> usize {
        *self as u8 as usize
    }

    pub const fn to_index_const(self) -> usize {
        self as u8 as usize
    }
}

impl Reaction {
    /// Given an optional target element (for swirl),
    /// return the DMG increase, piercing DMG, and an additional command.
    pub fn reaction_effects(&self, target: Option<Element>) -> (u8, u8, Option<Command>) {
        // TODO no proc effects for most reactions
        match self {
            Reaction::Bloom => (1, 0, Some(Command::ApplyStatusToTeam(StatusId::DendroCore))),
            Reaction::Burning => (1, 0, Some(Command::Summon(SummonId::BurningFlame))),
            Reaction::Crystallize => (1, 0, Some(Command::ApplyStatusToTeam(StatusId::CrystallizeShield))),
            Reaction::ElectroCharged => (1, 1, None),
            Reaction::Frozen => (1, 0, Some(Command::ApplyCharacterStatusToTarget(StatusId::Frozen))),
            Reaction::Melt => (2, 0, None),
            Reaction::Overloaded => (2, 0, Some(Command::ForceSwitchForTarget(RelativeCharIdx::Next))),
            Reaction::Quicken => (1, 0, Some(Command::ApplyStatusToTeam(StatusId::CatalyzingField))),
            Reaction::Superconduct => (1, 1, None),
            Reaction::Swirl => (0, 0, target.and_then(|t| t.swirl_effect())),
            Reaction::Vaporize => (2, 0, None),
        }
    }
}

pub fn find_reaction(elems: ElementSet, new_elem: Element) -> (ElementSet, Option<(Reaction, Option<Element>)>) {
    if elems.is_empty() {
        return (elems, None);
    }

    for elem in elems {
        if let (Some(r), te) = check_reaction(elem, new_elem) {
            let mut elems1 = elems;
            elems1.remove(elem);
            return (elems1, Some((r, te)));
        }
    }

    (elems, None)
}

#[cfg(test)]
mod tests {
    use crate::elem_set;

    use super::*;

    #[test]
    fn test_reaction_priority_cryo_before_dendro() {
        let es = elem_set![Element::Cryo, Element::Dendro];
        assert_eq!(
            (elem_set![Element::Dendro], Some((Reaction::Superconduct, None))),
            find_reaction(es, Element::Electro)
        );
        assert_eq!(
            (elem_set![Element::Dendro], Some((Reaction::Melt, None))),
            find_reaction(es, Element::Pyro)
        );
        assert_eq!(
            (elem_set![Element::Dendro], Some((Reaction::Frozen, None))),
            find_reaction(es, Element::Hydro)
        );
    }
}
