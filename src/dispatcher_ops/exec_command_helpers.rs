#![allow(non_snake_case)]
use enumset::{EnumSet, EnumSetType};

use crate::data_structures::CommandList;
use crate::tcg_model::enums::*;

use super::types::DispatchResult;
use crate::cmd_list;
use crate::types::status_impl::RespondsTo;
use crate::types::status_impl::StatusImpl;
use crate::zobrist_hash::game_state_mutation::PlayerHashContext;
use crate::{
    cards::ids::{lookup::*, *},
    types::{
        card_defs::*,
        command::*,
        deal_dmg::*,
        dice_counter::{distribution::*, DiceCounter},
        game_state::*,
    },
};

// TODO remove
#[derive(Debug, PartialOrd, Ord, EnumSetType)]
#[enumset(repr = "u8")]
pub enum CharIdx {
    I0 = 0,
    I1 = 1,
    I2 = 2,
    I3 = 3,
}

impl CharIdx {
    #[inline]
    pub fn value(self) -> u8 {
        self as isize as u8
    }
}

impl TryFrom<u8> for CharIdx {
    type Error = ();

    fn try_from(value: u8) -> Result<Self, Self::Error> {
        match value {
            0 => Ok(Self::I0),
            1 => Ok(Self::I1),
            2 => Ok(Self::I2),
            3 => Ok(Self::I3),
            _ => Err(()),
        }
    }
}

impl From<CharIdx> for u8 {
    #[inline]
    fn from(val: CharIdx) -> Self {
        val.value()
    }
}

pub type CharIdxSet = EnumSet<CharIdx>;

#[derive(Debug)]
pub enum ExecResult {
    Success,
    /// Suspend execution of commands and hand control back to the dispatcher.
    /// Then the dispatcher will return `suspended_state.get_dispatch_result()`
    Suspend(SuspendedState, Option<CommandList<(CommandContext, Command)>>),
    /// Stop executing commands and the dispatcher will return the specified result.
    Return(DispatchResult),
    /// Run additional commands before running the next command.
    AdditionalCmds(CommandList<(CommandContext, Command)>),
}

#[derive(Debug, Clone, Copy)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub enum RelativeCharIdx {
    Previous,
    Next,
    ClosestTo(u8),
    ImmediateNext,
}

#[macro_export]
macro_rules! view {
    ($p: ident) => {
        PlayerStateView {
            active_char_idx: $p.active_char_idx,
            char_states: &$p.char_states,
            flags: $p.flags,
            dice: $p.dice,
            // TODO will eventually be needed?
            // affected_by: $p.status_collection.get_affected_by_keys(),
            affected_by: Default::default(),
        }
    };
}

/// Mutate a particular `StatusCollection` within the `GameState` in the code block
/// while updating the Zobrish hash before and after the block.
///
/// Do not early terminate inside the block to preserve hash coherency.
#[macro_export]
macro_rules! mutate_statuses {
    ($self: expr, $player_id: expr, | $sc: ident | $closure: block) => {{
        let player_id: PlayerId = $player_id;
        let player: &mut _ = $self.players.get_mut(player_id);
        $crate::mutate_statuses_1!(phc!($self, player_id), player, |$sc| $closure)
    }};
}

/// Mutate a particular `StatusCollection` within the `GameState` in the code block
/// while updating the Zobrish hash before and after the block.
///
/// Do not early terminate inside the block to preserve hash coherency.
#[macro_export]
macro_rules! mutate_statuses_1 {
    ($c: expr, $player: ident, | $sc: ident | $closure: block) => {{
        let (h, player_id): $crate::zobrist_hash::game_state_mutation::PlayerHashContext = $c;
        let $sc: &mut $crate::types::game_state::StatusCollection = &mut $player.status_collection;
        $sc.zobrist_hash(h, player_id);
        let _r = $closure;
        $sc.zobrist_hash(h, player_id);
        _r
    }};
}

pub fn augment_outgoing_dmg_for_statuses(
    sc: &mut StatusCollection,
    sicb: StatusImplContextBuilder<DMGInfo>,
    dmg: &mut DealDMG,
) -> bool {
    sc.consume_statuses(
        sicb.src_char_idx_selector(),
        |si| si.responds_to().contains(RespondsTo::OutgoingDMG),
        |es, sk, si| si.outgoing_dmg(&sicb.build(sk, es), dmg),
    )
}

pub fn augment_outgoing_dmg_target_for_statuses(
    sc: &mut StatusCollection,
    sicb: StatusImplContextBuilder<DMGInfo>,
    tgt_chars: &CharStates,
    tgt_active_char_idx: u8,
    dmg: &DealDMG,
    target_char_idx: &mut u8,
) -> bool {
    sc.consume_statuses(
        sicb.src_char_idx_selector(),
        |si| si.responds_to().contains(RespondsTo::OutgoingDMG),
        |es, sk, si| {
            si.outgoing_dmg_target(
                &sicb.build(sk, es),
                tgt_chars,
                tgt_active_char_idx,
                dmg,
                target_char_idx,
            )
        },
    )
}

pub fn augment_late_outgoing_dmg_for_statuses(
    sc: &mut StatusCollection,
    sicb: StatusImplContextBuilder<DMGInfo>,
    dmg: &mut DealDMG,
) -> bool {
    sc.consume_statuses(
        sicb.src_char_idx_selector(),
        |si| si.responds_to().contains(RespondsTo::OutgoingDMG),
        |es, sk, si| si.late_outgoing_dmg(&sicb.build(sk, es), dmg),
    )
}

pub fn augment_outgoing_reaction_dmg_for_statuses(
    sc: &mut StatusCollection,
    sicb: StatusImplContextBuilder<DMGInfo>,
    reaction: (Reaction, Option<Element>),
    dmg: &mut DealDMG,
) -> bool {
    sc.consume_statuses(
        sicb.src_char_idx_selector(),
        |si| si.responds_to().contains(RespondsTo::OutgoingReactionDMG),
        |es, sk, si| si.outgoing_reaction_dmg(&sicb.build(sk, es), reaction, dmg),
    )
}

pub fn multiply_outgoing_dmg_for_statuses(
    sc: &mut StatusCollection,
    sicb: StatusImplContextBuilder<DMGInfo>,
    mult: &mut u8,
) -> bool {
    sc.consume_statuses(
        sicb.src_char_idx_selector(),
        |si| si.responds_to().contains(RespondsTo::MultiplyOutgoingDMG),
        |es, sk, si| si.multiply_dmg(&sicb.build(sk, es), mult),
    )
}

pub fn augment_incoming_dmg_for_statuses(
    sc: &mut StatusCollection,
    sicb: StatusImplContextBuilder,
    char_idx: u8,
    dmg: &mut DealDMG,
) -> bool {
    sc.consume_statuses(
        CharIdxSelector::One(char_idx),
        |si| si.responds_to().contains(RespondsTo::IncomingDMG),
        |es, sk, si| si.incoming_dmg(&sicb.build(sk, es), dmg),
    )
}

pub fn consume_shield_points_for_statuses(sc: &mut StatusCollection, char_idx: u8, dmg: &mut DealDMG) -> bool {
    let mut found = false;
    sc.for_each_char_status_mut_retain(
        Some(char_idx),
        |status_id, eff_state| {
            let status = status_id.get_status();
            if !(dmg.dmg > 0 && status.usages_as_shield_points) {
                return true;
            }
            found = true;
            let u = eff_state.get_usages();
            if u > dmg.dmg {
                let d = dmg.dmg;
                dmg.dmg = 0;
                eff_state.set_usages(u - d);
                true
            } else {
                // u <= dmg.dmg
                eff_state.set_usages(0);
                dmg.dmg -= u;
                status.manual_discard
            }
        },
        |_, _| {
            // Summons can't have Shield Points
            true
        },
        |_, _| {
            // Supports can't have Shield Points
            true
        },
    );
    found
}

impl CostType {
    #[inline]
    fn into_cmd_src(self, active_char_idx: u8) -> CommandSource {
        match self {
            CostType::Switching => CommandSource::Switch {
                from_char_idx: active_char_idx,
                dst_char_idx: active_char_idx,
            },
            CostType::Card(card_id) => CommandSource::Card { card_id, target: None },
            CostType::Skill(skill_id) => CommandSource::Skill {
                char_idx: active_char_idx,
                skill_id,
            },
        }
    }
}

// TODO can reduce cost for character talent cards
pub fn augment_cost(c: PlayerHashContext, player: &mut PlayerState, cost: &mut Cost, cost_type: CostType) -> bool {
    let char_idx = player.active_char_idx;
    if !player.status_collection.responds_to(RespondsTo::UpdateCost) {
        return false;
    }

    let view = &view!(player);
    mutate_statuses_1!(c, player, |sc| {
        let ctx = &CommandContext::EMPTY.with_src(cost_type.into_cmd_src(player.active_char_idx));
        let sicb = StatusImplContextBuilder::new(view, ctx, ());
        sc.consume_statuses(
            CharIdxSelector::One(char_idx),
            |si| si.responds_to().contains(RespondsTo::UpdateCost),
            |es, sk, si| si.update_cost(&sicb.build(sk, es), cost, cost_type),
        )
    })
}

pub fn augment_cost_immutable(player: &PlayerState, cost: &mut Cost, cost_type: CostType) {
    let sc = &player.status_collection;
    if !sc.responds_to(RespondsTo::UpdateCost) {
        return;
    }

    let char_idx = player.active_char_idx;
    let view = &view!(player);
    let ctx = &CommandContext::EMPTY.with_src(cost_type.into_cmd_src(player.active_char_idx));
    let sicb = StatusImplContextBuilder::new(view, ctx, ());
    sc.consume_statuses_immutable(
        CharIdxSelector::One(char_idx),
        |si| si.responds_to().contains(RespondsTo::UpdateCost),
        |es, sk, si| si.update_cost(&sicb.build(sk, es), cost, cost_type),
    );
}

pub fn update_gains_energy(player: &PlayerState, ctx_for_skill: &CommandContext, gains_energy: &mut bool) {
    let sc = &player.status_collection;
    if !sc.responds_to(RespondsTo::GainsEnergy) {
        return;
    }

    let char_idx = player.active_char_idx;
    let view = &view!(player);
    let ctx = &CommandContext::EMPTY;
    let sicb = StatusImplContextBuilder::new(view, ctx, ());
    sc.consume_statuses_immutable(
        CharIdxSelector::One(char_idx),
        |si| si.responds_to().contains(RespondsTo::GainsEnergy),
        |es, sk, si| {
            si.gains_energy(&sicb.build(sk, es), ctx_for_skill, gains_energy)
                .then_some(AppliedEffectResult::NoChange)
        },
    );
}

pub fn update_dice_distribution(player: &PlayerState, dist: &mut DiceDistribution) {
    let sc = &player.status_collection;
    if !sc.responds_to(RespondsTo::DiceDistribution) {
        return;
    }

    let view = &view!(player);
    let ctx = &CommandContext::EMPTY;
    let sicb = StatusImplContextBuilder::new(view, ctx, ());
    sc.consume_statuses_immutable(
        // Does not need to be active character to take effect
        CharIdxSelector::All,
        |si| si.responds_to().contains(RespondsTo::DiceDistribution),
        |es, sk, si| {
            si.dice_distribution(&sicb.build(sk, es), dist)
                .then_some(AppliedEffectResult::NoChange)
        },
    );
}

pub fn can_pay_dice_cost(player: &PlayerState, cost: &Cost, cost_type: CostType) -> bool {
    let ep = player.get_element_priority();
    let mut cost = *cost;
    augment_cost_immutable(player, &mut cost, cost_type);
    player.dice.try_pay_cost(&cost, &ep).is_some()
}

/// Assumption: augment_cost will never increase costs
pub fn try_pay_dice_cost(
    c: PlayerHashContext,
    player: &mut PlayerState,
    cost: &Cost,
    cost_type: CostType,
) -> Option<DiceCounter> {
    let ep = player.get_element_priority();
    if let Some(d) = player.dice.try_pay_cost(cost, &ep) {
        Some(d)
    } else {
        let mut cost = *cost;
        augment_cost(c, player, &mut cost, cost_type);
        player.dice.try_pay_cost(&cost, &ep)
    }
}

pub fn get_cast_skill_cmds(
    src_player: &PlayerState,
    ctx: &CommandContext,
    skill_id: SkillId,
) -> CommandList<(CommandContext, Command)> {
    let skill = skill_id.get_skill();
    let mut cmds: CommandList<(CommandContext, Command)> = cmd_list![];
    if let Some(deal_dmg) = skill.deal_dmg {
        cmds.push((*ctx, Command::DealDMG(deal_dmg)));
    }

    if let Some(status_id) = skill.apply {
        match status_id.get_status().attach_mode {
            StatusAttachMode::Character => {
                let char_idx = src_player.active_char_idx;
                cmds.push((*ctx, Command::ApplyCharacterStatus(status_id, char_idx.into())));
            }
            StatusAttachMode::Team => {
                cmds.push((*ctx, Command::ApplyStatusToTeam(status_id)));
            }
            StatusAttachMode::Summon => panic!("Cannot attach summon status {status_id:?}."),
            StatusAttachMode::Support => panic!("Cannot attach support status {status_id:?}."),
        }
    }

    if let Some(summon_spec) = skill.summon {
        match summon_spec {
            SummonSpec::One(summon_id) => {
                cmds.push((*ctx, Command::Summon(summon_id)));
            }
            SummonSpec::MultiRandom { count: 0, .. } => {}
            SummonSpec::MultiRandom {
                summon_ids,
                count,
                prioritize_new,
            } => {
                let existing_summon_ids = if prioritize_new {
                    src_player
                        .status_collection
                        .iter_entries()
                        .filter_map(|k| match k.key {
                            StatusKey::Summon(summon_id) => Some(summon_id),
                            _ => None,
                        })
                        .fold(Default::default(), |s, k| s | k)
                } else {
                    Default::default()
                };
                cmds.push((
                    *ctx,
                    Command::SummonRandom(SummonRandomSpec::new(summon_ids, existing_summon_ids, count)),
                ));
            }
        }
    }

    for c in skill.commands.to_vec() {
        cmds.push((*ctx, c));
    }

    let mut gains_energy = !skill.no_energy;
    update_gains_energy(src_player, ctx, &mut gains_energy);
    if let Some(si) = skill.skill_impl {
        si.get_commands(src_player, ctx, &mut cmds);
    }

    if gains_energy && skill.skill_type != SkillType::ElementalBurst {
        cmds.push((*ctx, Command::AddEnergy(1, CmdCharIdx::Active)));
    }

    cmds.push((
        *ctx,
        Command::TriggerXEvent(XEvent::Skill(XEventSkill {
            src_player_id: ctx.src_player_id,
            src_char_idx: src_player.active_char_idx,
            skill_id,
        })),
    ));
    cmds.push((*ctx, Command::HandOverPlayer));
    cmds
}

impl CommandSource {
    #[inline]
    pub(crate) fn selected_char_idx_or(&self, or_char_idx: u8) -> u8 {
        match self {
            CommandSource::Card {
                target: Some(CardSelection::OwnCharacter(c)),
                ..
            } => *c,
            _ => or_char_idx,
        }
    }
}

impl RelativeCharIdx {
    pub(crate) fn indexing_seq(self, char_idx: u8, n: u8) -> impl Iterator<Item = u8> {
        let i0 = char_idx;
        (0..n).map(move |d| match self {
            RelativeCharIdx::Previous => {
                if d > n {
                    n
                } else {
                    (i0 + n - d - 1) % n
                }
            }
            RelativeCharIdx::Next => {
                if d > n {
                    n
                } else {
                    (i0 + d + 1) % n
                }
            }
            RelativeCharIdx::ClosestTo(mid) => {
                if mid >= n {
                    return n - 1 - d;
                }
                let dr = n - 1 - mid;
                let rev = d % 2 == 1;
                let k = 2 * dr.min(mid);
                let low = d <= k;
                if mid == 0 {
                    d
                } else if mid == n - 1 {
                    n - 1 - d
                } else if low {
                    let d = (d + 1) / 2;
                    if rev {
                        mid - d
                    } else {
                        mid + d
                    }
                } else if mid >= dr {
                    n - 1 - d
                } else {
                    d
                }
            }
            RelativeCharIdx::ImmediateNext => {
                if d == 0 {
                    (i0 + 1).min(n - 1)
                } else {
                    i0
                }
            }
        })
    }
}

impl CharStates {
    #[inline]
    pub(crate) fn relative_switch_char_idx(&self, active_char_idx: u8, switch_type: RelativeCharIdx) -> Option<u8> {
        switch_type
            .indexing_seq(active_char_idx, self.len())
            .find(|&j| self.is_valid_char_idx(j))
    }

    pub(crate) fn get_taken_most_dmg(&self) -> Option<(u8, &CharState)> {
        self.enumerate_valid().max_by_key(|(_, c)| c.get_total_dmg_taken())
    }
}

impl PlayerState {
    #[inline]
    pub(crate) fn relative_switch_char_idx(&self, switch_type: RelativeCharIdx) -> Option<u8> {
        self.char_states
            .relative_switch_char_idx(self.active_char_idx, switch_type)
    }
}

#[cfg(test)]
mod tests {
    use proptest::prelude::*;

    use super::RelativeCharIdx;

    const N: u8 = 15;

    fn arb_relative_switch_type_expect_immediate_next() -> impl Strategy<Value = RelativeCharIdx> {
        prop_oneof![
            Just(RelativeCharIdx::Previous),
            Just(RelativeCharIdx::Next),
            (0..N).prop_map(RelativeCharIdx::ClosestTo),
        ]
    }
    proptest! {
        #[test]
        fn indexing_seq_has_length_of_n(n in 1..N, s in arb_relative_switch_type_expect_immediate_next(), char_idx in 0..N) {
            prop_assume!(char_idx < n);
            assert_eq!(n, s.indexing_seq(char_idx, n).collect::<Vec<_>>().len() as u8);
        }

        #[test]
        fn indexing_seq_is_a_permutation_of_range_from_zero_to_n(n in 1..N, s in arb_relative_switch_type_expect_immediate_next(), char_idx in 0..N) {
            prop_assume!(char_idx < n);
            let perm = s.indexing_seq(char_idx, N).collect::<Vec<_>>();
            let sorted = { let mut sorted = perm.clone(); sorted.sort(); sorted };
            assert_eq!((0..N).collect::<Vec<_>>(), sorted, "{perm:?}, s={s:?} char_idx={char_idx}");
        }

        #[test]
        fn indexing_seq_for_closest_to(char_idx in 0..N) {
            let perm = RelativeCharIdx::ImmediateNext.indexing_seq(char_idx, N).take(2).collect::<Vec<_>>();
            if char_idx + 1 < N {
                assert_eq!(vec![char_idx + 1, char_idx], perm);
            } else {
                assert_eq!(vec![char_idx, char_idx], perm);
            }
        }
    }
}
