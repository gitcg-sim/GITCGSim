use super::applied_effect_state::AppliedEffectState;
use super::card_defs::{Card, CardType, Skill};
use super::dice_counter::DiceCounter;
use super::enums::EquipSlot;
use super::game_state::*;
use super::{deal_dmg::DealDMG, enums::Element, game_state::PlayerId};
use crate::cards::ids::*;
use crate::data_structures::capped_list::CappedLengthList8;
use crate::data_structures::CommandList;
pub use crate::dispatcher_ops::exec_command_helpers::RelativeCharIdx;
use enumset::{EnumSet, EnumSetType};

mod command_context;

// TODO refactor away EventId way of triggering events
#[allow(clippy::upper_case_acronyms)]
pub mod xevent {
    use enumset::EnumSet;

    use crate::tcg_model::{
        deal_dmg::DealDMGType,
        enums::{Reaction, SkillType},
    };

    use super::*;

    #[derive(Debug, Clone, Copy)]
    #[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
    pub struct XEventDMG {
        pub src_player_id: PlayerId,
        pub tgt_char_idx: u8,
        pub dmg_value: u8,
        pub dmg_type: DealDMGType,
        pub dmg_info: DMGInfo,
        pub reaction: Option<(Reaction, Option<Element>)>,
        pub defeated: bool,
    }

    #[derive(Debug, Clone, Copy)]
    #[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
    pub struct XEventSkill {
        pub src_player_id: PlayerId,
        pub src_char_idx: u8,
        pub skill_id: SkillId,
    }

    impl XEventSkill {
        #[inline]
        pub fn skill_type(&self) -> SkillType {
            self.skill_id.get_skill().skill_type
        }
    }

    enum XEventCodeBuilder {
        DMG {
            from_self: bool,
            is_reaction: bool,
            defeated: bool,
        },
        Skill {
            from_self: bool,
            skill_type: SkillType,
        },
    }

    /// 16 possible values
    #[derive(Debug, EnumSetType, Default)]
    #[allow(non_camel_case_types)]
    pub enum XEventCode {
        #[default]
        DMG_Opp_NR_ND,
        DMG_Opp_NR_D,
        DMG_Opp_R_ND,
        DMG_Opp_R_D,
        DMG_Self_NR_ND,
        DMG_Self_NR_D,
        DMG_Self_R_ND,
        DMG_Self_R_D,
        Skill_Self_NA,
        Skill_Self_Skill,
        Skill_Self_Burst,
        Skill_Opp_NA,
        Skill_Opp_Skill,
        Skill_Opp_Burst,
    }

    impl XEventCodeBuilder {
        #[inline]
        fn build(&self) -> XEventCode {
            match *self {
                Self::DMG {
                    from_self,
                    is_reaction,
                    defeated,
                } => match (from_self, is_reaction, defeated) {
                    (false, false, false) => XEventCode::DMG_Opp_NR_ND,
                    (false, false, true) => XEventCode::DMG_Opp_NR_D,
                    (false, true, false) => XEventCode::DMG_Opp_R_ND,
                    (false, true, true) => XEventCode::DMG_Opp_R_D,
                    (true, false, false) => XEventCode::DMG_Self_NR_ND,
                    (true, false, true) => XEventCode::DMG_Self_NR_D,
                    (true, true, false) => XEventCode::DMG_Self_R_ND,
                    (true, true, true) => XEventCode::DMG_Self_R_D,
                },
                Self::Skill { from_self, skill_type } => match (from_self, skill_type) {
                    (false, SkillType::NormalAttack) => XEventCode::Skill_Opp_NA,
                    (false, SkillType::ElementalSkill) => XEventCode::Skill_Opp_Skill,
                    (false, SkillType::ElementalBurst) => XEventCode::Skill_Opp_Burst,
                    (true, SkillType::NormalAttack) => XEventCode::Skill_Self_NA,
                    (true, SkillType::ElementalSkill) => XEventCode::Skill_Self_Skill,
                    (true, SkillType::ElementalBurst) => XEventCode::Skill_Self_Burst,
                },
            }
        }
    }

    #[derive(Debug, Clone, Copy)]
    #[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
    pub enum XEvent {
        DMG(XEventDMG),
        Skill(XEventSkill),
    }

    impl XEvent {
        #[inline]
        pub fn code(&self, player_id: PlayerId) -> XEventCode {
            match self {
                Self::DMG(dmg) => XEventCodeBuilder::DMG {
                    from_self: dmg.src_player_id == player_id,
                    is_reaction: dmg.reaction.is_some(),
                    defeated: dmg.defeated,
                }
                .build(),
                Self::Skill(skill) => XEventCodeBuilder::Skill {
                    from_self: skill.src_player_id == player_id,
                    skill_type: skill.skill_id.get_skill().skill_type,
                }
                .build(),
            }
        }

        #[inline]
        pub fn mask(&self, player_id: PlayerId) -> XEventMask {
            let mut bs = EnumSet::<XEventCode>::default();
            bs.insert(self.code(player_id));
            bs
        }
    }

    pub type XEventMask = EnumSet<XEventCode>;

    #[allow(dead_code)]
    pub mod xevent_mask {
        use super::{XEventCode, XEventMask};
        use enumset::enum_set;

        pub const NONE: XEventMask = enum_set![];

        pub const DMG: XEventMask = enum_set![
            XEventCode::DMG_Opp_NR_ND
                | XEventCode::DMG_Opp_NR_D
                | XEventCode::DMG_Opp_R_ND
                | XEventCode::DMG_Opp_R_D
                | XEventCode::DMG_Self_NR_ND
                | XEventCode::DMG_Self_NR_D
                | XEventCode::DMG_Self_R_ND
                | XEventCode::DMG_Self_R_D
        ];

        pub const DMG_INCOMING: XEventMask = enum_set![
            XEventCode::DMG_Opp_NR_ND | XEventCode::DMG_Opp_NR_D | XEventCode::DMG_Opp_R_ND | XEventCode::DMG_Opp_R_D
        ];

        pub const DMG_OUTGOING: XEventMask = enum_set![
            XEventCode::DMG_Self_NR_ND
                | XEventCode::DMG_Self_NR_D
                | XEventCode::DMG_Self_R_ND
                | XEventCode::DMG_Self_R_D
        ];

        pub const DMG_REACTION: XEventMask = enum_set![
            XEventCode::DMG_Opp_R_ND | XEventCode::DMG_Opp_R_D | XEventCode::DMG_Self_R_ND | XEventCode::DMG_Self_R_D
        ];

        pub const DMG_DEFEAT: XEventMask = enum_set![
            XEventCode::DMG_Opp_NR_D | XEventCode::DMG_Opp_R_D | XEventCode::DMG_Self_NR_D | XEventCode::DMG_Self_R_D
        ];

        pub const SKILL_FROM_SELF: XEventMask =
            enum_set![XEventCode::Skill_Self_NA | XEventCode::Skill_Self_Skill | XEventCode::Skill_Self_Burst];

        pub const SKILL_FROM_OPP: XEventMask =
            enum_set![XEventCode::Skill_Opp_NA | XEventCode::Skill_Opp_Skill | XEventCode::Skill_Opp_Burst];

        pub const SKILL_NA: XEventMask = enum_set![XEventCode::Skill_Self_NA | XEventCode::Skill_Opp_NA];

        pub const SKILL_SKILL: XEventMask = enum_set![XEventCode::Skill_Self_Skill | XEventCode::Skill_Opp_Skill];

        pub const SKILL_BURST: XEventMask = enum_set![XEventCode::Skill_Self_Burst | XEventCode::Skill_Opp_Burst];
    }
}

pub use command_context::*;

pub use xevent::*;

#[derive(Debug, PartialOrd, Ord, EnumSetType)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
#[enumset(repr = "u8")]
pub enum EventId {
    EndPhase,
    EndOfTurn,
    Switched,
    StartOfActionPhase,
    /// "Before you choose your action:"
    BeforeAction,
    /// "When you declare the end of your Round:"
    DeclareEndOfRound,
}

#[derive(Debug)]
pub struct TriggerEventContext<'a, 'b, 'c, 'd, 'v, E = EventId> {
    pub c: StatusImplContext<'a, 'b, 'c, 'v>,
    pub status_key: StatusKey,
    pub event_id: E,
    pub ctx_for_dmg: &'c CommandContext,
    pub out_cmds: &'d mut CommandList<(CommandContext, Command)>,
}

#[derive(Debug)]
pub struct StatusImplContext<'a, 'b, 'c, 'v, D = ()> {
    pub src_player_state: &'v PlayerStateView<'a>,
    pub status_key: StatusKey,
    pub eff_state: &'b AppliedEffectState,
    pub ctx: &'c CommandContext,
    pub dmg: D,
}

impl<'a, 'b, 'c, 'v, D> StatusImplContext<'a, 'b, 'c, 'v, D> {
    #[inline]
    pub fn new(
        src_player_state: &'v PlayerStateView<'a>,
        status_key: StatusKey,
        eff_state: &'b AppliedEffectState,
        ctx: &'c CommandContext,
        dmg: D,
    ) -> Self {
        Self {
            src_player_state,
            status_key,
            eff_state,
            ctx,
            dmg,
        }
    }
}

pub struct StatusImplContextBuilder<'a, 'c, 'v, D = ()> {
    pub src_player_state: &'v PlayerStateView<'a>,
    pub ctx: &'c CommandContext,
    pub dmg: D,
}

impl<'a, 'c, 'v, D: Copy> StatusImplContextBuilder<'a, 'c, 'v, D> {
    pub fn new(src_player_state: &'v PlayerStateView<'a>, ctx: &'c CommandContext, dmg: D) -> Self {
        Self {
            src_player_state,
            ctx,
            dmg,
        }
    }

    pub fn build<'b>(
        &self,
        status_key: StatusKey,
        eff_state: &'b AppliedEffectState,
    ) -> StatusImplContext<'a, 'b, 'c, 'v, D> {
        let Self {
            src_player_state,
            dmg,
            ctx,
        } = *self;
        StatusImplContext::new(src_player_state, status_key, eff_state, ctx, dmg)
    }

    #[inline]
    pub fn src_char_idx_selector(&self) -> CharIdxSelector {
        self.ctx.src.char_idx().into()
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub enum CostType {
    Switching,
    Card(CardId),
    Skill(SkillId),
}

impl CostType {
    #[inline]
    pub fn get_skill(&self) -> Option<&'static Skill> {
        match self {
            Self::Skill(skill_id) => Some(skill_id.get_skill()),
            _ => None,
        }
    }

    #[inline]
    pub fn get_card(&self) -> Option<&'static Card> {
        match self {
            Self::Card(card_id) => Some(card_id.get_card()),
            _ => None,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct SummonRandomSpec {
    pub summon_ids: CappedLengthList8<SummonId>,
    pub existing_summon_ids: EnumSet<SummonId>,
    pub count: u8,
}

impl SummonRandomSpec {
    pub fn new(summon_ids: CappedLengthList8<SummonId>, existing_summon_ids: EnumSet<SummonId>, count: u8) -> Self {
        Self {
            summon_ids,
            existing_summon_ids,
            count,
        }
    }
}

/// A command represents a unit of effect performed on the game state.
/// Most elements of card text or game mechanics are translated into `Command`s.
/// Common examples of commands are:
///  - Deal DMG
///  - Add energy to active character
///  - Gain a character/team status
///  - Create a summon
///  - Switch character
///
/// In addition, triggering effects are performed through `Command`s as well.
/// When a command refers to "active character", it is the source player's active character
/// OR the character chosen by the player by card's targeting.
/// See also: `CommandContext`.
#[derive(Debug, Clone, Copy, Default)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub enum Command {
    #[default]
    Nop,
    /// Cast a skill without paying cost or validating if it can be casted.
    /// This command is for implementing "Cast skill" commands.
    CastSkill(SkillId),
    // TODO deprecate this
    TriggerEvent(EventId),
    TriggerXEvent(XEvent),
    SwitchCharacter(u8),
    /// Apply element to the skill's caster OR the active character.
    ApplyElementToSelf(Element),
    DealDMG(DealDMG),
    DealDMGRelative(DealDMG, RelativeCharIdx),
    TakeDMG(DealDMG),
    TakeDMGForAffectedBy(StatusId, DealDMG),
    DealSwirlDMG(Element, u8),
    /// Heal the active (or selected) character.
    Heal(u8),
    /// Heall all of the player's characters.
    HealAll(u8),
    /// Heal the character that has taken the most DMG.
    HealTakenMostDMG(u8),
    /// Add energy to selected character of the player.
    AddEnergy(u8),
    /// Add energy to one character without maximum energy (active prioritized.)
    AddEnergyWithoutMaximum(u8),
    /// Add energy to character by index in the 2nd parameter.
    AddEnergyToCharacter(u8, u8),
    AddEnergyToNonActiveCharacters(u8),
    /// Add energy to selected character of the player.
    SetEnergy(u8),
    /// For "Calx's Arts"
    ShiftEnergy,
    /// Increase the Usages/Duration/Shield Points of a status or summon.
    /// The status must exist or else nothing happens. The usages/duration can go above the max stacks.
    IncreaseStatusUsages(StatusKey, u8),
    /// Delete a status for command source.
    DeleteStatus(StatusKey),
    /// Delete a status for command target player.
    DeleteStatusForTarget(StatusKey),
    RerollDice,
    /// Add Elemental Dice to the player.
    AddDice(DiceCounter),
    /// Sub Elemental Dice from the player's dice pool.
    SubtractDice(DiceCounter),
    AddCardsToHand(CappedLengthList8<CardId>),
    DrawCards(u8, Option<CardType>),
    /// Apply a status to the player's character by index.
    ApplyStatusToCharacter(StatusId, u8),
    ApplyStatusToActiveCharacter(StatusId),
    ApplyEquipmentToCharacter(EquipSlot, StatusId, u8),
    ApplyTalentToCharacter(u8, Option<StatusId>),
    /// Apply a character status state to target player's active character. Completes ignores context.
    ApplyCharacterStatusToActive(PlayerId, StatusId, AppliedEffectState),
    AddSupport(SupportSlot, SupportId),
    /// Apply a team status to the player.
    ApplyStatusToTeam(StatusId),
    /// Apply a character status to the command target character
    ApplyStatusToTarget(StatusId),
    /// Apply a character status to all opponent characters
    ApplyStatusToAllOpponentCharacters(StatusId),
    /// Apply a team status to the command target's player
    ApplyStatusToTargetTeam(StatusId),
    /// Create or refresh a summon on the player.
    Summon(SummonId),
    /// Summon random given count and existing summons to deprioritize
    SummonRandom(SummonRandomSpec),
    SwitchPrev,
    SwitchNext,
    /// Force the target player to switch the active character to the specified relative switch.
    ForceSwitchForTarget(RelativeCharIdx),
    /// Hand turn to the next player. This command is used for performing Combat Actions.
    HandOverPlayer,
    /// End the turn and perform end of turn actions.
    EndOfTurn,
    /// Implements Keqing's "Stellar Restoration" skill:
    ///  - "creates 1 Lightning Stiletto"
    ///  - "When Keqing uses Stellar Restoration with this card (Lightning Stiletto) in Hand: ..."
    StellarRestorationFromSkill,
}
