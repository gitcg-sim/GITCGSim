use crate::std_subset::cmp::min;

use crate::{
    cards::ids::*,
    data_structures::{CommandList, List8},
    list8,
};

use super::{
    card_impl::CardImpl,
    command::{Command, CommandContext},
    game_state::{PlayerState, StatusCollection},
    tcg_model::*,
};

/// Specifications for a character card's passive effect, which applies a status on duel start.
#[derive(Debug, Clone, Copy)]
pub struct Passive {
    pub name: &'static str,
    pub apply_statuses: List8<StatusId>,
}

impl Passive {
    pub const fn new(name: &'static str) -> Self {
        Self {
            name,
            apply_statuses: list8![],
        }
    }

    pub const fn new_hidden() -> Self {
        Self {
            name: "",
            apply_statuses: list8![],
        }
    }

    pub const fn status(self, status_id: StatusId) -> Self {
        Self {
            apply_statuses: list8![status_id],
            ..self
        }
    }

    pub const fn statuses(self, apply_statuses: List8<StatusId>) -> Self {
        Self { apply_statuses, ..self }
    }
}

/// Specifications for a character card.
#[derive(Debug, Clone, Copy)]
pub struct CharCard {
    pub name: &'static str,
    pub elem: Element,
    pub weapon: WeaponType,
    pub faction: Faction,
    pub max_health: u8,
    pub max_energy: u8,
    pub skills: List8<SkillId>,
    pub passive: Option<Passive>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub enum SupportType {
    Companion,
    Location,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub enum CardType {
    Event,
    Food,
    ElementalResonance(Element),
    Support(SupportType),
    Weapon(WeaponType),
    Artifact,
    Talent(CharId),
}

/// Specifications for a non-character card:
///  - Equipment Card (Weapon/Artifact/Talent)
///  - Action Card (Support/Food/Event)
pub struct Card {
    pub name: &'static str,
    pub cost: Cost,
    /// Card effects as a list of commands. Will only be used by the default
    /// implementation of `CardImpl`.
    pub effects: List8<Command>,
    pub card_type: CardType,
    pub card_impl: Option<&'static dyn CardImpl>,
}

#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, PartialOrd, Ord, Hash)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct Cost {
    pub elem_cost: Option<(Element, u8)>,
    pub unaligned_cost: u8,
    pub aligned_cost: u8,
    pub energy_cost: u8,
}

impl Cost {
    pub const ZERO: Cost = Cost {
        elem_cost: None,
        unaligned_cost: 0,
        aligned_cost: 0,
        energy_cost: 0,
    };

    pub const ONE: Cost = Cost {
        elem_cost: None,
        unaligned_cost: 0,
        aligned_cost: 1,
        energy_cost: 0,
    };

    pub const fn elem(elem: Element, cost: u8) -> Cost {
        Cost {
            elem_cost: Some((elem, cost)),
            unaligned_cost: 0,
            aligned_cost: 0,
            energy_cost: 0,
        }
    }

    pub const fn unaligned(unaligned_cost: u8) -> Cost {
        Cost {
            elem_cost: None,
            unaligned_cost,
            aligned_cost: 0,
            energy_cost: 0,
        }
    }

    pub const fn aligned(aligned_cost: u8) -> Cost {
        Cost {
            elem_cost: None,
            unaligned_cost: 0,
            aligned_cost,
            energy_cost: 0,
        }
    }

    pub const fn with_unaligned(self, unaligned_cost: u8) -> Cost {
        Cost { unaligned_cost, ..self }
    }

    pub const fn with_energy(self, energy_cost: u8) -> Cost {
        Cost { energy_cost, ..self }
    }

    #[inline]
    pub fn total_dice(&self) -> u8 {
        self.elem_cost.map(|x| x.1).unwrap_or_default() + self.unaligned_cost + self.aligned_cost
    }

    #[inline]
    pub fn try_reduce_by(&mut self, mut value: u8) -> bool {
        if value == 0 {
            return true;
        }
        for r in [&mut self.unaligned_cost, &mut self.aligned_cost] {
            if *r > 0 {
                let d = min(value, *r);
                *r -= d;
                value -= d;
            }
            if value == 0 {
                return true;
            }
        }
        let Some((_, cost)) = &mut self.elem_cost else {
            return false;
        };
        if *cost == 0 {
            return false;
        }
        *cost -= min(value, *cost);
        true
    }

    #[inline]
    pub fn try_reduce_unaligned_cost(&mut self, value: u8) -> bool {
        if value == 0 {
            return true;
        }

        if self.unaligned_cost == 0 {
            false
        } else if self.unaligned_cost >= value {
            self.unaligned_cost -= value;
            true
        } else {
            self.unaligned_cost = 0;
            true
        }
    }

    #[inline]
    pub fn try_reduce_elemental_cost(&mut self, value: u8, elem: Element) -> bool {
        if value == 0 {
            return true;
        }

        let Some((e, cost)) = &mut self.elem_cost else {
            return false;
        };
        if *e == elem && *cost >= value {
            *cost -= value;
            true
        } else {
            false
        }
    }
}

pub trait SkillImpl {
    #[allow(unused_variables)]
    fn get_commands(
        &self,
        src_player: &PlayerState,
        status_collection: &StatusCollection,
        ctx: &CommandContext,
        cmds: &mut CommandList<(CommandContext, Command)>,
    ) {
    }
}

#[derive(Clone, Copy, Debug)]
pub struct PrepareSkill {
    pub turns: u8,
    pub skill_id: SkillId,
}

impl PrepareSkill {
    pub const fn new(turns: u8, skill_id: SkillId) -> Self {
        Self { turns, skill_id }
    }
}

/// The commands generated by a `Skill` will be added in the following order:
/// `deal_dmg, apply, summon, commands, skill_impl`
#[derive(Clone, Copy)]
pub struct Skill {
    pub name: &'static str,
    pub skill_type: SkillType,
    pub cost: Cost,
    /// Skill grants no energy. This field is ignored for Elemental Bursts
    pub no_energy: bool,
    pub deal_dmg: Option<DealDMG>,
    pub apply: Option<StatusId>,
    pub summon: Option<SummonSpec>,
    pub commands: List8<Command>,
    pub skill_impl: Option<&'static dyn SkillImpl>,
}

impl Skill {
    pub const fn new() -> Self {
        Self {
            name: "",
            skill_type: SkillType::NormalAttack,
            cost: Cost::ZERO,
            no_energy: false,
            deal_dmg: None,
            apply: None,
            summon: None,
            commands: list8![],
            skill_impl: None,
        }
    }
}

impl Default for Skill {
    fn default() -> Self {
        Self::new()
    }
}

impl crate::std_subset::fmt::Debug for Skill {
    fn fmt(&self, f: &mut crate::std_subset::fmt::Formatter<'_>) -> crate::std_subset::fmt::Result {
        f.debug_struct("Skill")
            .field("name", &self.name)
            .field("skill_type", &self.skill_type)
            .field("cost", &self.cost)
            .field("no_energy", &self.no_energy)
            .field("deal_dmg", &self.deal_dmg)
            .field("apply", &self.apply)
            .field("summon", &self.summon)
            .field("commands", &self.commands)
            .field("skill_impl", &self.skill_impl.map(|_| ()))
            .finish()
    }
}

#[derive(Debug, Clone, Copy)]
pub enum SummonSpec {
    One(SummonId),
    /// Summon multiple, prioritizing new ones
    MultiRandom {
        summon_ids: List8<SummonId>,
        count: u8,
        prioritize_new: bool,
    },
}

#[derive(Debug, Clone, Copy)]
pub struct CounterSpec {
    pub name: &'static str,
    pub default_value: u8,
    pub resets_at_turn_end: bool,
}

impl CounterSpec {
    pub const fn new(name: &'static str, default_value: u8) -> Self {
        Self {
            name,
            default_value,
            resets_at_turn_end: false,
        }
    }

    pub const fn resets_at_turn_end(self, resets_at_turn_end: bool) -> Self {
        Self {
            resets_at_turn_end,
            ..self
        }
    }
}

/// Definition for an applied effect for status/summon/support
#[derive(Debug, Clone, Copy)]
pub struct Status {
    pub name: &'static str,
    pub attach_mode: StatusAttachMode,
    /// The number of usages granted for "Usages: " status OR the number of "Shield Points".
    pub usages: Option<u8>,
    /// The duration of the "Duration: (Rounds)" status.
    pub duration_rounds: Option<u8>,
    /// Max. number of stacks a "Usages" status can have. None to be equal to starting usages.
    pub max_stacks: Option<u8>,
    /// Usages count as Shield Points.
    pub usages_as_shield_points: bool,
    /// Don't automatically discard on usage/duration exhaustion.
    pub manual_discard: bool,
    pub counter_spec: Option<CounterSpec>,
    /// Usages/Duration/Shield Points increase when the specified character has talent equipped.
    pub talent_usages_increase: Option<(CharId, u8)>,
    /// Intended to be applied on opponent team instead of own team.
    pub applies_to_opposing: bool,
    /// Shifts to the next active character when the attached character dies.
    pub shifts_to_next_active_on_death: bool,
    /// Applies a new status to the attached character when this status is removed.
    pub reapplies_on_discard: Option<StatusId>,
    pub casted_by_character: Option<CharId>,
}

impl Status {
    pub const EMPTY: Status = Status {
        name: "",
        attach_mode: StatusAttachMode::Character,
        usages: None,
        duration_rounds: None,
        max_stacks: None,
        usages_as_shield_points: false,
        manual_discard: false,
        counter_spec: None,
        talent_usages_increase: None,
        applies_to_opposing: false,
        shifts_to_next_active_on_death: false,
        reapplies_on_discard: None,
        casted_by_character: None,
    };

    pub const fn new_indef(name: &'static str, attach_mode: StatusAttachMode) -> Status {
        Status {
            name,
            attach_mode,
            ..Self::EMPTY
        }
    }

    pub const fn new_usages(
        name: &'static str,
        attach_mode: StatusAttachMode,
        usages: u8,
        max_stacks: Option<u8>,
    ) -> Status {
        Status {
            name,
            attach_mode,
            usages: Some(usages),
            max_stacks,
            ..Self::EMPTY
        }
    }

    pub const fn new_summon_usages(name: &'static str, usages: u8) -> Status {
        Status {
            name,
            attach_mode: StatusAttachMode::Summon,
            usages: Some(usages),
            ..Self::EMPTY
        }
    }

    pub const fn new_shield_points(
        name: &'static str,
        attach_mode: StatusAttachMode,
        shield_points: u8,
        max_stacks: Option<u8>,
    ) -> Status {
        Status {
            name,
            attach_mode,
            usages: Some(shield_points),
            usages_as_shield_points: true,
            max_stacks,
            ..Self::EMPTY
        }
    }

    pub const fn new_duration(name: &'static str, attach_mode: StatusAttachMode, duration_rounds: u8) -> Status {
        Status {
            name,
            attach_mode,
            duration_rounds: Some(duration_rounds),
            ..Self::EMPTY
        }
    }

    pub const fn prepare_skill(self, prepare_for_turns: u8) -> Self {
        self.manual_discard(true).counter(CounterSpec {
            name: "[Prepare Skill]",
            default_value: prepare_for_turns - 1,
            resets_at_turn_end: false,
        })
    }

    pub const fn manual_discard(self, manual_discard: bool) -> Self {
        Status { manual_discard, ..self }
    }

    pub const fn counter(self, counter_spec: CounterSpec) -> Self {
        Status {
            counter_spec: Some(counter_spec),
            ..self
        }
    }

    pub const fn talent_usages_increase(self, char_id: CharId, value: u8) -> Self {
        Status {
            talent_usages_increase: Some((char_id, value)),
            ..self
        }
    }

    pub const fn shield_points(self, shield_points: u8) -> Self {
        Status {
            attach_mode: StatusAttachMode::Character,
            usages_as_shield_points: true,
            usages: Some(shield_points),
            max_stacks: None,
            ..self
        }
    }

    pub const fn applies_to_opposing(self) -> Self {
        Status {
            applies_to_opposing: true,
            ..self
        }
    }

    pub const fn shifts_to_next_active_on_death(self) -> Self {
        Status {
            shifts_to_next_active_on_death: true,
            ..self
        }
    }

    pub const fn reapplies_on_discard(self, status_id: StatusId) -> Self {
        Status {
            reapplies_on_discard: Some(status_id),
            ..self
        }
    }

    pub const fn casted_by_character(self, char_id: CharId) -> Self {
        Status {
            casted_by_character: Some(char_id),
            ..self
        }
    }

    #[inline]
    pub fn get_casted_by_char_id(&self) -> CharId {
        self.casted_by_character
            .or_else(|| self.talent_usages_increase.map(|t| t.0))
            .unwrap_or_else(|| panic!("Must declare casted_by_character for status: {}", self.name))
    }
}

impl crate::std_subset::fmt::Display for CardType {
    fn fmt(&self, f: &mut crate::std_subset::fmt::Formatter<'_>) -> crate::std_subset::fmt::Result {
        match *self {
            CardType::Event => write!(f, "Event"),
            CardType::Food => write!(f, "Food"),
            CardType::ElementalResonance(elem) => write!(f, "Elemental Resonance: {elem}"),
            CardType::Support(t) => write!(f, "Support: {t}"),
            CardType::Weapon(t) => write!(f, "Weapon: {t}"),
            CardType::Artifact => write!(f, "Artifact"),
            CardType::Talent(char_id) => write!(f, "Talent: {}", char_id.get_char_card().name),
        }
    }
}

crate::impl_display_from_debug!(SupportType);
