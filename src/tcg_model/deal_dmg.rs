use crate::tcg_model::enums::Element;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub enum DealDMGType {
    Piercing,
    Physical,
    Elemental(Element),
}

impl DealDMGType {
    pub const PYRO: DealDMGType = DealDMGType::Elemental(Element::Pyro);
    pub const HYDRO: DealDMGType = DealDMGType::Elemental(Element::Hydro);
    pub const CRYO: DealDMGType = DealDMGType::Elemental(Element::Cryo);
    pub const ELECTRO: DealDMGType = DealDMGType::Elemental(Element::Electro);
    pub const DENDRO: DealDMGType = DealDMGType::Elemental(Element::Dendro);
    pub const GEO: DealDMGType = DealDMGType::Elemental(Element::Geo);
    pub const ANEMO: DealDMGType = DealDMGType::Elemental(Element::Anemo);

    #[inline]
    pub(crate) fn is_elemental(&self) -> bool {
        self.element().is_some()
    }

    #[inline]
    pub(crate) fn element(&self) -> Option<Element> {
        match self {
            Self::Piercing => None,
            Self::Physical => None,
            Self::Elemental(elem) => Some(*elem),
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct DealDMG {
    pub dmg_type: DealDMGType,
    pub dmg: u8,
    pub piercing_dmg_to_standby: u8,
}

impl DealDMG {
    #[inline]
    pub const fn new(dmg_type: DealDMGType, dmg: u8, piercing_dmg_to_standby: u8) -> DealDMG {
        DealDMG {
            dmg_type,
            dmg,
            piercing_dmg_to_standby,
        }
    }

    #[inline]
    pub const fn new_piercing(piercing_dmg: u8) -> DealDMG {
        DealDMG {
            dmg_type: DealDMGType::Piercing,
            dmg: piercing_dmg,
            piercing_dmg_to_standby: 0,
        }
    }

    #[inline]
    /// Convert physical DMG into elemental DMG.
    /// Returns: true if infusion changed this DealDMG instance, false otherwise
    pub fn infuse(&mut self, deal_dmg_type: DealDMGType) -> bool {
        if let DealDMGType::Physical = self.dmg_type {
            self.dmg_type = deal_dmg_type;
            return true;
        }
        false
    }

    #[inline]
    pub fn reduce(&mut self, value: u8) -> bool {
        if value == 0 || self.dmg == 0 {
            return false;
        }
        self.dmg = self.dmg - crate::std_subset::cmp::min(self.dmg, value);
        true
    }

    #[inline]
    pub fn try_reduce<T>(&mut self, value: u8, if_some: T) -> Option<T> {
        self.reduce(value).then_some(if_some)
    }
}
