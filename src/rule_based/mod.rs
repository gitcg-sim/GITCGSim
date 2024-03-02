use crate::std_subset::cmp::min;

use rand::{rngs::SmallRng, Rng, SeedableRng};
use smallvec::SmallVec;

use crate::{
    cards::ids::*,
    data_structures::ActionList,
    game_state_wrapper::*,
    reaction::check_reaction,
    tcg_model::*,
    types::{
        game_state::{GameState, PlayerId},
        input::{Input, PlayerAction},
        nondet::*,
    },
};

#[derive(Debug)]
pub struct RuleBasedSearchConfig {
    /// Switch score contribution based on a character's HP. Encourages switching away from characters at low HP
    pub switch_scores_hp: [u8; 10],
    /// Switch score contribution when a character's energy is full
    pub switch_score_burst: u8,
    /// Switch score contribution when a character's energy is 1 short of full
    pub switch_score_one_off_burst: u8,
    pub switch_score_reaction: u8,
    pub switch_score_diff_threshold: u8,
    pub switch_score_diff_threshold_opp_ended_round: u8,
    pub switch_score_offensive: u8,
    pub switch_score_defensive: u8,
    pub defensive_low_hp_threshold: u8,
    pub end_round_score: u8,
    pub normal_attack_score: u8,
    pub elemental_skill_score: u8,
    pub elemental_burst_score: u8,
    pub elemental_tuning_score: u8,
    pub play_card_min_dice_for_skills: u8,
    pub score_candidate_threshold: u8,
}

impl Default for RuleBasedSearchConfig {
    fn default() -> Self {
        Self::DEFAULT
    }
}

impl GameState {
    pub fn has_outgoing_reaction(&self, player_id: PlayerId, src_char_idx: u8, tgt_char_idx: u8) -> bool {
        let Some(src_char) = self.get_player(player_id).try_get_character(src_char_idx) else {
            return false;
        };
        let Some(tgt_char) = self.get_player(player_id.opposite()).try_get_character(tgt_char_idx) else {
            return false;
        };
        let src_elem = src_char.char_id.get_char_card().elem;
        tgt_char
            .applied
            .iter()
            .any(|tgt_elem| check_reaction(src_elem, tgt_elem).0.is_some())
    }

    pub fn opponent_can_counter_switch(&self, player_id: PlayerId, src_char_idx: Option<u8>) -> bool {
        let is_fast_action = src_char_idx
            .map(|src_char_idx| self.check_switch_is_fast_action(player_id, src_char_idx))
            .unwrap_or_default();
        let opp_ended_round = self.phase.opponent_ended_round(player_id);
        !(is_fast_action || opp_ended_round)
    }
}

impl RuleBasedSearchConfig {
    pub const DEFAULT: Self = Self {
        switch_scores_hp: [0, 1, 2, 4, 6, 8, 10, 11, 12, 12],
        switch_score_burst: 5,
        switch_score_one_off_burst: 2,
        switch_score_reaction: 6,
        switch_score_diff_threshold: 15,
        switch_score_diff_threshold_opp_ended_round: 5,
        switch_score_offensive: 10,
        switch_score_defensive: 30,
        end_round_score: 1,
        normal_attack_score: 4,
        elemental_skill_score: 20,
        elemental_burst_score: 30,
        elemental_tuning_score: 5,
        play_card_min_dice_for_skills: 6,
        score_candidate_threshold: 0,
        defensive_low_hp_threshold: 3,
    };

    pub fn switch_scores<S: NondetState>(
        &self,
        position: &GameStateWrapper<S>,
        player_id: PlayerId,
    ) -> SmallVec<[u8; 4]> {
        let game_state = &position.game_state;
        let (src_player, opp_player) = (
            game_state.get_player(player_id),
            game_state.get_player(player_id.opposite()),
        );
        let own_dice_count = src_player.dice.total();
        // let opp_dice_count = game_state.get_player(player_id.opposite()).dice.total();
        let (tgt_char_idx, tgt_hp) = (opp_player.active_char_idx, opp_player.get_active_character().get_hp());
        let mut scores = src_player
            .char_states
            .iter_all()
            .enumerate()
            .map(|(src_char_idx, src_char)| {
                if src_char.is_invalid() {
                    return 0;
                }
                let src_char_idx = src_char_idx as u8;
                let has_outgoing_reaction = game_state.has_outgoing_reaction(player_id, src_char_idx, tgt_char_idx);
                let has_incoming_reaction =
                    game_state.has_outgoing_reaction(player_id.opposite(), tgt_char_idx, src_char_idx);
                let offensive_threat_score = {
                    let can_switch_attack = own_dice_count > 3;
                    let can_counter_switch = src_char_idx == !src_player.active_char_idx
                        || game_state.opponent_can_counter_switch(player_id, Some(src_char_idx));
                    let has_offensive_reaction = can_switch_attack && has_outgoing_reaction && !can_counter_switch;
                    let has_kill =
                        can_switch_attack && tgt_hp <= self.defensive_low_hp_threshold && !can_counter_switch;
                    if has_offensive_reaction || has_kill {
                        self.switch_score_offensive
                    } else {
                        0
                    }
                };
                let has_burst = src_char.get_energy() >= src_char.char_id.get_char_card().max_energy;
                let has_one_off_burst = src_char.get_energy() + 1 == src_char.char_id.get_char_card().max_energy;
                let hp = crate::std_subset::cmp::min(
                    src_char.get_hp().saturating_sub(1) as usize,
                    self.switch_scores_hp.len(),
                );
                let offensive_part = offensive_threat_score
                    + (has_outgoing_reaction as u8) * self.switch_score_reaction
                    + (has_burst as u8) * self.switch_score_burst
                    + (has_one_off_burst as u8) * self.switch_score_one_off_burst
                    + self.switch_scores_hp[hp];

                let incoming_threat_score = {
                    let src_hp = src_player.get_active_character().get_hp();
                    if has_incoming_reaction || src_hp <= self.defensive_low_hp_threshold {
                        self.switch_score_defensive
                    } else {
                        0
                    }
                };
                let defensive_part = incoming_threat_score + (has_incoming_reaction as u8) * self.switch_score_reaction;
                offensive_part.saturating_sub(defensive_part)
            })
            .collect::<SmallVec<[u8; 4]>>();
        let threshold = if game_state.phase.opponent_ended_round(player_id) {
            self.switch_score_diff_threshold_opp_ended_round
        } else {
            self.switch_score_diff_threshold
        };
        let subtract = scores[src_player.active_char_idx as usize] + threshold;
        scores.iter_mut().for_each(|s| *s = s.saturating_sub(subtract));
        scores
    }

    pub fn play_card_score<S: NondetState>(
        &self,
        position: &GameStateWrapper<S>,
        player_id: PlayerId,
        card_id: CardId,
    ) -> u8 {
        let player = position.game_state.get_player(player_id);
        let cost_total = card_id.get_card().cost.total_dice();
        let dice_total = player.dice.total();
        let hand_size = player.hand.len();
        if cost_total > 0 && dice_total >= cost_total && dice_total - cost_total < self.play_card_min_dice_for_skills {
            0
        } else {
            10 * min(hand_size, 5) / 5
        }
    }

    pub fn cast_skill_score<S: NondetState>(&self, _: &GameStateWrapper<S>, _: PlayerId, skill_id: SkillId) -> u8 {
        match skill_id.get_skill().skill_type {
            SkillType::NormalAttack => self.normal_attack_score,
            SkillType::ElementalSkill => self.elemental_skill_score,
            SkillType::ElementalBurst => self.elemental_burst_score,
        }
    }

    pub fn action_scores<S: NondetState>(
        &self,
        position: &GameStateWrapper<S>,
        actions: &ActionList<Input>,
        player_id: PlayerId,
    ) -> ActionList<(Input, u8)> {
        let switch_scores = self.switch_scores(position, player_id);
        let player = position.game_state.get_player(player_id);
        let on_dice_count = player.dice[Dice::Omni]
            + player.dice[Dice::Elem(player.get_active_character().char_id.get_char_card().elem)];
        actions
            .iter()
            .map(|&input| {
                let Input::FromPlayer(_, action) = input else {
                    return (input, 0);
                };
                let score = match action {
                    PlayerAction::EndRound => self.end_round_score,
                    PlayerAction::PlayCard(card_id, _) => self.play_card_score(position, player_id, card_id),
                    PlayerAction::ElementalTuning(_) => {
                        if on_dice_count <= 2 {
                            self.elemental_tuning_score
                        } else {
                            0
                        }
                    }
                    PlayerAction::CastSkill(skill_id) => self.cast_skill_score(position, player_id, skill_id),
                    PlayerAction::SwitchCharacter(i) => switch_scores[i as usize],
                    PlayerAction::PostDeathSwitch(i) => switch_scores[i as usize],
                };
                (input, score)
            })
            .collect()
    }
}

#[derive(Debug)]
pub struct RuleBasedSearch {
    pub config: RuleBasedSearchConfig,
    pub rng: SmallRng,
}

impl RuleBasedSearch {
    pub fn new(config: RuleBasedSearchConfig) -> Self {
        Self {
            config,
            rng: SmallRng::seed_from_u64(10),
        }
    }

    pub fn search_and_select<S: NondetState>(
        &mut self,
        position: &GameStateWrapper<S>,
        maximize_player: PlayerId,
    ) -> (ActionList<(Input, u8)>, usize) {
        let mut action_scores = self
            .config
            .action_scores(position, &position.actions(), maximize_player);
        action_scores.sort_by_key(|(_, k)| *k);
        let i = {
            let rng = &mut self.rng;
            let max_score = action_scores[action_scores.len() - 1].1;
            let t = self.config.score_candidate_threshold;
            let min_score = if max_score >= t { max_score - t } else { 0 };
            let last = action_scores.len() - 1;
            let idxs: SmallVec<[usize; 4]> = action_scores
                .iter()
                .enumerate()
                .filter_map(|(i, (_, s))| if i == last || *s >= min_score { Some(i) } else { None })
                .collect();
            idxs[rng.gen_range(0..idxs.len())]
        };
        (action_scores, i)
    }
}
