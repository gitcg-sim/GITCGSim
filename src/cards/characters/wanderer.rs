use super::*;

pub const C: CharCard = CharCard {
    name: "Wanderer",
    elem: Element::Anemo,
    weapon: WeaponType::Catalyst,
    faction: Faction::Other,
    max_health: 10,
    max_energy: 3,
    skills: list8![
        SkillId::YuubanMeigen,
        SkillId::HanegaSongOfTheWind,
        SkillId::KyougenFiveCeremonialPlays,
    ],
    passive: None,
};

pub const YUUBAN_MEIGEN: Skill = skill_na("Yuuban Meigen", Element::Anemo, 1, DealDMGType::ANEMO);

pub const HANEGA_SONG_OF_THE_WIND: Skill = Skill {
    name: "Hanega: Song of the Wind",
    skill_type: SkillType::ElementalSkill,
    cost: cost_elem(Element::Anemo, 3, 0, 0),
    deal_dmg: Some(deal_elem_dmg(Element::Anemo, 2, 0)),
    apply: Some(StatusId::Windfavored),
    ..Skill::new()
};

pub const KYOUGEN_FIVE_CEREMONIAL_PLAYS: Skill = Skill {
    name: "Kyougen: Five Ceremonial Plays",
    skill_type: SkillType::ElementalBurst,
    cost: cost_elem(Element::Anemo, 3, 0, 3),
    skill_impl: Some(&KyougenFiveCeremonialPlaysImpl()),
    ..Skill::new()
};

pub struct KyougenFiveCeremonialPlaysImpl();
impl SkillImpl for KyougenFiveCeremonialPlaysImpl {
    fn get_commands(
        &self,
        src_player: &PlayerState,
        ctx: &CommandContext,
        cmds: &mut CommandList<(CommandContext, Command)>,
    ) {
        let char_idx = src_player.active_char_idx;
        if src_player
            .status_collection
            .has_character_status(char_idx, StatusId::Windfavored)
        {
            cmds.push((*ctx, Command::DealDMG(deal_elem_dmg(Element::Anemo, 8, 0))));
            cmds.push((
                *ctx,
                Command::DeleteStatus(StatusKey::Character(char_idx, StatusId::Windfavored)),
            ));
        } else {
            cmds.push((*ctx, Command::DealDMG(deal_elem_dmg(Element::Anemo, 7, 0))));
        }
    }
}

pub mod windfavored {
    use super::*;

    pub const S: Status = Status::new_usages("Windfavored", StatusAttachMode::Character, 2, None);

    decl_status_impl_type!(Windfavored, I);
    impl StatusImpl for Windfavored {
        fn responds_to(&self) -> EnumSet<RespondsTo> {
            enum_set![RespondsTo::OutgoingDMG | RespondsTo::OutgoingDMGTarget]
        }

        fn outgoing_dmg(&self, e: &StatusImplContext<DMGInfo>, dmg: &mut DealDMG) -> Option<AppliedEffectResult> {
            if !e.is_normal_attack() {
                return None;
            }
            // TODO next opposing character
            dmg.dmg += 2;
            Some(AppliedEffectResult::ConsumeUsage)
        }

        fn outgoing_dmg_target(
            &self,
            e: &StatusImplContext<DMGInfo>,
            tgt_chars: &CharStates,
            tgt_active_char_idx: u8,
            _: &DealDMG,
            tgt_char_idx: &mut u8,
        ) -> Option<AppliedEffectResult> {
            if !e.is_normal_attack() {
                return None;
            }
            let Some(char_idx1) = tgt_chars.relative_switch_char_idx(tgt_active_char_idx, RelativeCharIdx::Next) else {
                return None;
            };
            *tgt_char_idx = char_idx1;
            Some(AppliedEffectResult::NoChange)
        }
    }
}
