use enumset::EnumSetType;

#[allow(clippy::derived_hash_with_manual_eq)]
#[derive(Debug, PartialOrd, Ord, Hash, EnumSetType)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub enum Element {
    Pyro = 0,
    Hydro = 1,
    Cryo = 2,
    Electro = 3,
    Dendro = 4,
    Geo = 5,
    Anemo = 6,
}

impl Element {
    pub const VALUES: [Element; 7] = [
        Element::Pyro,
        Element::Hydro,
        Element::Cryo,
        Element::Electro,
        Element::Dendro,
        Element::Geo,
        Element::Anemo,
    ];

    pub fn get_name(&self) -> &'static str {
        match self {
            Element::Pyro => "Pyro",
            Element::Hydro => "Hydro",
            Element::Cryo => "Cryo",
            Element::Electro => "Electro",
            Element::Dendro => "Dendro",
            Element::Geo => "Geo",
            Element::Anemo => "Anemo",
        }
    }
}

#[allow(clippy::derived_hash_with_manual_eq)]
#[derive(Debug, PartialOrd, Ord, Hash, EnumSetType)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub enum Reaction {
    Bloom,
    Burning,
    Crystallize,
    ElectroCharged,
    Frozen,
    Melt,
    Overloaded,
    Quicken,
    Superconduct,
    Swirl,
    Vaporize,
}

#[derive(Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub enum Dice {
    // Omni can be used to pay any 1 dice cost
    Omni,
    // Dice of a particular element
    Elem(Element),
}

impl std::fmt::Debug for Dice {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Omni => write!(f, "Omni"),
            Self::Elem(e) => write!(f, "E.{e:?}"),
        }
    }
}

impl Dice {
    pub const PYRO: Dice = Dice::Elem(Element::Pyro);
    pub const HYDRO: Dice = Dice::Elem(Element::Hydro);
    pub const CRYO: Dice = Dice::Elem(Element::Cryo);
    pub const ELECTRO: Dice = Dice::Elem(Element::Electro);
    pub const DENDRO: Dice = Dice::Elem(Element::Dendro);
    pub const GEO: Dice = Dice::Elem(Element::Geo);
    pub const ANEMO: Dice = Dice::Elem(Element::Anemo);

    #[inline]
    pub fn to_index(&self) -> usize {
        match self {
            Self::Omni => 0,
            Self::Elem(e) => e.to_index() + 1,
        }
    }
}

#[allow(clippy::derived_hash_with_manual_eq)]
#[derive(Debug, PartialOrd, Ord, Hash, EnumSetType)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub enum WeaponType {
    Other,
    Bow,
    Catalyst,
    Claymore,
    Polearm,
    Sword,
}

#[allow(clippy::derived_hash_with_manual_eq)]
#[derive(Debug, PartialOrd, Ord, Hash, EnumSetType)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub enum Faction {
    Mondstadt,
    Liyue,
    Inazuma,
    Sumeru,
    Monster,
    Fatui,
    Hilichurl,
    Kairagi,
}

#[allow(clippy::derived_hash_with_manual_eq)]
#[derive(Debug, Default, PartialOrd, Ord, Hash, EnumSetType)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub enum SkillType {
    #[default]
    NormalAttack,
    ElementalSkill,
    ElementalBurst,
}

#[allow(clippy::derived_hash_with_manual_eq)]
#[derive(Debug, PartialOrd, Ord, Hash, EnumSetType)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub enum StatusAttachMode {
    Character,
    Team,
    Summon,
    Support,
}

#[allow(clippy::derived_hash_with_manual_eq)]
#[derive(Debug, PartialOrd, Ord, Hash, EnumSetType)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub enum EquipSlot {
    Artifact,
    Weapon,
    Talent,
}
