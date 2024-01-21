use enumset::{enum_set, EnumSet};

use crate::cards::ids::*;
use crate::data_structures::CommandList;
use crate::list8;
use crate::types::{
    card_defs::*, card_impl::*, command::*, dice_counter::*, game_state::*, status_impl::*, tcg_model::*,
    StatusSpecModifier,
};

pub mod artifact;
pub mod talent;
pub mod weapon;

#[macro_export]
macro_rules! card_impl_for_artifact {
    (@status $self: expr, $status_id: expr) => { $status_id };
    (@status $self: expr $(,)?) => { ($self).status_id };
    ($type: ident $(, $status_id: expr)?) => {
        impl CardImpl for $type {
            fn selection(&self) -> Option<CardSelectionSpec> {
                Some(CardSelectionSpec::OwnCharacter)
            }

            fn get_effects(
                &self,
                cic: &CardImplContext,
                ctx: &CommandContext,
                commands: &mut CommandList<(CommandContext, Command)>,
            ) {
                if let Some(CardSelection::OwnCharacter(char_idx)) = cic.selection {
                    commands.push((
                        *ctx,
                        Command::ApplyEquipment(
                            EquipSlot::Artifact,
                            card_impl_for_artifact!(@status self, $($status_id)?),
                            char_idx.into()
                        ),
                    ))
                } else {
                    panic!("Invalid selection")
                }
            }
        }
    };
}

pub const fn equipment_status(name: &'static str) -> Status {
    Status {
        name,
        attach_mode: StatusAttachMode::Character,
        manual_discard: true,
        ..Status::EMPTY
    }
}

pub struct ElementalArtifact {
    pub elem: Element,
    pub status_id: StatusId,
    pub dice_guarantee: Option<u8>,
}

impl StatusImpl for ElementalArtifact {
    fn responds_to(&self) -> EnumSet<RespondsTo> {
        if self.dice_guarantee.is_some() {
            enum_set![RespondsTo::UpdateCost | RespondsTo::DiceDistribution]
        } else {
            enum_set![RespondsTo::UpdateCost]
        }
    }

    fn update_cost(&self, e: &StatusImplContext, cost: &mut Cost, cost_type: CostType) -> Option<AppliedEffectResult> {
        if !e.eff_state.can_use_once_per_round() {
            return None;
        }

        // TODO also check equip talent cost
        let CostType::Skill(_) = cost_type else { return None };
        cost.try_reduce_elemental_cost(1, self.elem)
            .then_some(AppliedEffectResult::ConsumeOncePerRound)
    }

    fn dice_distribution(&self, _: &StatusImplContext, dist: &mut DiceDistribution) -> bool {
        let Some(count) = self.dice_guarantee else { return false };
        dist.guarantee_elem(self.elem, count)
    }
}

card_impl_for_artifact!(ElementalArtifact);

fn get_effects_for_weapon(
    status_id: StatusId,
    cic: &CardImplContext,
    ctx: &CommandContext,
    commands: &mut CommandList<(CommandContext, Command)>,
) {
    if let Some(CardSelection::OwnCharacter(char_idx)) = cic.selection {
        commands.push((
            *ctx,
            Command::ApplyEquipment(EquipSlot::Weapon, status_id, char_idx.into()),
        ));
    } else {
        panic!("Weapon card: Invalid selection")
    }
}

fn can_be_played_for_weapon(weapon_type: WeaponType, cic: &CardImplContext) -> CanBePlayedResult {
    if let Some(CardSelection::OwnCharacter(char_idx)) = cic.selection {
        if !cic
            .game_state
            .get_player(cic.active_player_id)
            .is_valid_char_idx(char_idx)
        {
            return CanBePlayedResult::InvalidSelection;
        }
        let card = cic
            .game_state
            .get_player(cic.active_player_id)
            .get_character_card(char_idx);
        if card.weapon != weapon_type {
            return CanBePlayedResult::InvalidSelection;
        }
    } else {
        return CanBePlayedResult::InvalidSelection;
    }
    CanBePlayedResult::CanBePlayed
}

#[macro_export]
macro_rules! card_impl_for_weapon {
    ($name: ident) => {
        impl CardImpl for $name {
            fn can_be_played(&self, cic: &CardImplContext) -> CanBePlayedResult {
                can_be_played_for_weapon(self.weapon_type, cic)
            }

            fn selection(&self) -> Option<CardSelectionSpec> {
                Some(CardSelectionSpec::OwnCharacter)
            }

            fn get_effects(
                &self,
                cic: &CardImplContext,
                ctx: &CommandContext,
                commands: &mut CommandList<(CommandContext, Command)>,
            ) {
                get_effects_for_weapon(self.status_id, cic, ctx, commands);
            }
        }
    };
}

pub struct Weapon2 {
    pub weapon_type: WeaponType,
    pub status_id: StatusId,
}

impl StatusImpl for Weapon2 {
    fn responds_to(&self) -> EnumSet<RespondsTo> {
        enum_set![RespondsTo::OutgoingDMG]
    }

    fn outgoing_dmg(&self, e: &StatusImplContext<DMGInfo>, dmg: &mut DealDMG) -> Option<AppliedEffectResult> {
        if e.skill_id().is_some() {
            dmg.dmg += 1;
            return Some(AppliedEffectResult::NoChange);
        }
        None
    }
}

card_impl_for_weapon!(Weapon2);

pub struct SacrificialWeapon {
    pub weapon_type: WeaponType,
    pub status_id: StatusId,
}

impl StatusImpl for SacrificialWeapon {
    fn responds_to(&self) -> EnumSet<RespondsTo> {
        enum_set![RespondsTo::OutgoingDMG | RespondsTo::TriggerXEvent]
    }

    fn responds_to_events(&self) -> XEventMask {
        xevent_mask::SKILL_FROM_SELF & xevent_mask::SKILL_SKILL
    }

    fn outgoing_dmg(&self, e: &StatusImplContext<DMGInfo>, dmg: &mut DealDMG) -> Option<AppliedEffectResult> {
        if e.skill_id().is_some() {
            dmg.dmg += 1;
            return Some(AppliedEffectResult::NoChange);
        }
        None
    }

    fn trigger_xevent(&self, e: &mut TriggerEventContext<XEvent>) -> Option<AppliedEffectResult> {
        let SkillType::ElementalSkill = e.get_event_skill_ensuring_attached_character()?.skill_type() else {
            return None;
        };

        if !e.c.eff_state.can_use_once_per_round() {
            return None;
        }

        let Some(c) = e.c.src_char_card() else { return None };
        e.out_cmds
            .push((*e.ctx_for_dmg, Command::AddDice(DiceCounter::elem(c.elem, 1))));
        Some(AppliedEffectResult::ConsumeOncePerRound)
    }
}

card_impl_for_weapon!(SacrificialWeapon);

pub struct FavoniusWeapon {
    pub weapon_type: WeaponType,
    pub status_id: StatusId,
}

impl StatusImpl for FavoniusWeapon {
    fn responds_to(&self) -> EnumSet<RespondsTo> {
        enum_set![RespondsTo::OutgoingDMG | RespondsTo::TriggerXEvent]
    }

    fn responds_to_events(&self) -> XEventMask {
        xevent_mask::SKILL_FROM_SELF & xevent_mask::SKILL_SKILL
    }

    fn outgoing_dmg(&self, e: &StatusImplContext<DMGInfo>, dmg: &mut DealDMG) -> Option<AppliedEffectResult> {
        let Some(_) = e.skill() else { return None };
        dmg.dmg += 1;
        Some(AppliedEffectResult::NoChange)
    }

    fn trigger_xevent(&self, e: &mut TriggerEventContext<XEvent>) -> Option<AppliedEffectResult> {
        let SkillType::ElementalSkill = e.get_event_skill_ensuring_attached_character()?.skill_type() else {
            return None;
        };
        e.add_cmd(Command::AddEnergy(1, CmdCharIdx::Active));
        Some(AppliedEffectResult::NoChange)
    }
}

card_impl_for_weapon!(FavoniusWeapon);

pub struct SkywardWeapon {
    pub weapon_type: WeaponType,
    pub status_id: StatusId,
}

impl StatusImpl for SkywardWeapon {
    fn responds_to(&self) -> EnumSet<RespondsTo> {
        enum_set![RespondsTo::OutgoingDMG]
    }

    fn outgoing_dmg(&self, e: &StatusImplContext<DMGInfo>, dmg: &mut DealDMG) -> Option<AppliedEffectResult> {
        let Some(skill) = e.skill() else { return None };
        dmg.dmg += 1;
        if e.eff_state.can_use_once_per_round() && SkillType::NormalAttack == skill.skill_type {
            dmg.dmg += 1;
            Some(AppliedEffectResult::ConsumeOncePerRound)
        } else {
            Some(AppliedEffectResult::NoChange)
        }
    }
}

card_impl_for_weapon!(SkywardWeapon);

pub struct Talent {
    pub skill_id: Option<SkillId>,
    pub status_id: Option<StatusId>,
}

impl CardImpl for Talent {
    fn can_be_played(&self, cic: &CardImplContext) -> CanBePlayedResult {
        let CardType::Talent(expected_char_id) = cic.card.card_type else {
            panic!("CardImpl for Talent: Not a talent card")
        };

        let Some(CardSelection::OwnCharacter(char_idx)) = cic.selection else {
            return CanBePlayedResult::InvalidSelection;
        };

        let player = cic.game_state.players.get(cic.active_player_id);
        let Some(char_state) = player.try_get_character(char_idx) else {
            return CanBePlayedResult::InvalidSelection;
        };

        // To be able to play talent card, the target must be the expected character
        if expected_char_id != char_state.char_id {
            return CanBePlayedResult::InvalidSelection;
        }

        if self.skill_id.is_some() {
            // To be able to cast skill, the target must be the active character
            if char_idx != player.active_char_idx {
                CanBePlayedResult::CannotBePlayed
            } else {
                CanBePlayedResult::CanBePlayed
            }
        } else {
            CanBePlayedResult::CanBePlayed
        }
    }

    fn selection(&self) -> Option<CardSelectionSpec> {
        Some(CardSelectionSpec::OwnCharacter)
    }

    fn get_effects(
        &self,
        cic: &CardImplContext,
        ctx: &CommandContext,
        commands: &mut CommandList<(CommandContext, Command)>,
    ) {
        let CardType::Talent(..) = cic.card.card_type else {
            panic!("CardImpl for Talent: Not a talent card")
        };

        let Some(CardSelection::OwnCharacter(char_idx)) = cic.selection else {
            panic!("CardImpl for Talent: Invalid selection")
        };

        commands.push((*ctx, Command::ApplyTalent(self.status_id, char_idx.into())));
        if let Some(skill_id) = self.skill_id {
            commands.push((
                ctx.with_src(CommandSource::Skill { char_idx, skill_id }),
                Command::CastSkill(skill_id),
            ))
        }
    }
}
