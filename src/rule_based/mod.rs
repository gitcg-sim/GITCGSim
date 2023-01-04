use std::cmp::min;

use rand::{rngs::SmallRng, Rng, SeedableRng};
use smallvec::SmallVec;

use crate::{
    cards::ids::{CardId, GetCard, GetCharCard, GetSkill, SkillId},
    data_structures::ActionList,
    game_tree_search::*,
    linked_list,
    reaction::check_reaction,
    tcg_model::enums::*,
    types::{
        game_state::PlayerId,
        input::{Input, PlayerAction},
        nondet::*,
        ElementSet,
    },
};

#[derive(Debug)]
pub struct RuleBasedSearchConfig {
    pub switch_scores_for_hp: [u8; 10],
    pub switch_score_for_burst: u8,
    pub switch_score_reaction: u8,
    pub switch_score_diff_threshold: u8,
    pub switch_score_opponent_ended_round: u8,
    pub switch_score_diff_threshold_opp_ended_round: u8,
    pub end_round_score: u8,
    pub normal_attack_score: u8,
    pub elemental_skill_score: u8,
    pub elemental_burst_score: u8,
    pub elemental_tuning_score: u8,
    pub min_dice_for_skills: u8,
    pub score_candidate_threshold: u8,
}

impl Default for RuleBasedSearchConfig {
    fn default() -> Self {
        Self::DEFAULT
    }
}

impl RuleBasedSearchConfig {
    pub const DEFAULT: Self = Self {
        switch_scores_for_hp: [0, 1, 3, 4, 6, 8, 9, 9, 10, 10],
        switch_score_for_burst: 2,
        switch_score_reaction: 3,
        switch_score_opponent_ended_round: 1,
        switch_score_diff_threshold: 3,
        switch_score_diff_threshold_opp_ended_round: 2,
        end_round_score: 1,
        normal_attack_score: 4,
        elemental_skill_score: 6,
        elemental_burst_score: 10,
        elemental_tuning_score: 2,
        min_dice_for_skills: 7,
        score_candidate_threshold: 0,
    };

    pub fn switch_scores<S: NondetState>(
        &self,
        position: &GameStateWrapper<S>,
        player_id: PlayerId,
    ) -> SmallVec<[u8; 4]> {
        fn have_reaction(attack_elem: Element, applied: ElementSet) -> bool {
            applied.iter().any(|elem| check_reaction(attack_elem, elem).0.is_some())
        }

        let hps = &self.switch_scores_for_hp;
        let Self {
            switch_score_for_burst: sb,
            switch_score_reaction: r,
            ..
        } = *self;
        let opp_ended_round_score = if position.game_state.phase.opponent_ended_round(player_id) {
            self.switch_score_opponent_ended_round
        } else {
            0
        };
        let opp_active_char_idx = position.game_state.get_player(player_id.opposite()).active_char_index;
        let opp_chars = &position.game_state.get_player(player_id.opposite()).char_states;
        let opp_active_char = &opp_chars[opp_active_char_idx as usize];
        let opp_elem = opp_active_char.char_id.get_char_card().elem;
        let opp_applied = opp_active_char.applied;
        let active_char_idx = position.game_state.get_player(player_id).active_char_index;
        let own_chars = &position.game_state.get_player(player_id).char_states;
        let mut scores: SmallVec<[u8; 4]> = own_chars
            .iter()
            .map(|char_state| {
                let char_card = char_state.char_id.get_char_card();
                let burst_score = if char_state.get_energy() >= char_card.max_energy {
                    sb
                } else {
                    0
                };
                let own_elem = char_card.elem;
                let own_applied = char_state.applied;
                let own_can_react = have_reaction(own_elem, opp_applied);
                let opp_can_react = have_reaction(opp_elem, own_applied);
                let hp_score = self.switch_scores_for_hp[min(hps.len() - 1, char_state.get_hp() as usize)];
                opp_ended_round_score + hp_score + (if own_can_react { r } else { 0 })
                    - (if opp_can_react { r } else { 0 })
                    + burst_score
            })
            .collect();
        let cur = scores[active_char_idx as usize];
        for score in &mut scores {
            *score = if *score <= cur { 0 } else { *score - cur };
        }
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
        if cost_total > 0 && dice_total >= cost_total && dice_total - cost_total < self.min_dice_for_skills {
            0
        } else {
            10 * min(hand_size as u8, 5) / 5
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
                return (input, 0)
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
}

impl<S: NondetState> GameTreeSearch<GameStateWrapper<S>> for RuleBasedSearch {
    fn search(
        &mut self,
        position: &GameStateWrapper<S>,
        maximize_player: PlayerId,
    ) -> SearchResult<GameStateWrapper<S>> {
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
        let mut counter = SearchCounter::default();
        counter.states_visited += 1;
        SearchResult {
            pv: linked_list![action_scores[i].0],
            eval: Default::default(),
            counter,
        }
    }
}
