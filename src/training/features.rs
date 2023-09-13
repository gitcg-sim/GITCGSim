use crate::cards::ids::{CardId, GetCard, GetSkill};
use crate::impl_as_slice;
use crate::prelude::Input;
use crate::status_impls::prelude::*;
use crate::types::card_defs::CardType;
use crate::types::input::PlayerAction;
use crate::zobrist_hash::CHAR_COUNT;
use crate::{
    game_tree_search::GameStateWrapper,
    types::{
        dice_counter::{DiceCounter, ElementPriority},
        game_state::*,
        nondet::NondetState,
    },
};
use serde::{Deserialize, Serialize};

pub const N_CHARS: usize = 3;

struct DiceSummary {
    pub omni_count: u8,
    pub active_count: u8,
    pub important_count: u8,
    pub off_count: u8,
}

impl DiceSummary {
    #[inline(always)]
    pub fn from_dice_counter(dice: &DiceCounter, ep: &ElementPriority) -> Self {
        let omni_count = dice.omni;
        let mut active_count = 0;
        let mut important_count = 0;
        let mut off_count = 0;
        for e in Element::VALUES {
            if Some(e) == ep.active_elem {
                active_count += dice[Dice::Elem(e)];
            } else if ep.important_elems.contains(e) {
                important_count += dice[Dice::Elem(e)];
            } else {
                off_count += dice[Dice::Elem(e)];
            }
        }
        Self {
            omni_count,
            active_count,
            important_count,
            off_count,
        }
    }
}

pub trait FeatureEntry: Copy {
    type Output;
    fn make_entry<F: Fn() -> String>(self, get_name: F, value: f32) -> Self::Output;
}

#[derive(Copy, Clone)]
pub struct FeatureName;
impl FeatureEntry for FeatureName {
    type Output = String;

    #[inline(always)]
    fn make_entry<F: Fn() -> String>(self, get_name: F, _: f32) -> Self::Output {
        get_name()
    }
}

#[derive(Copy, Clone)]
pub struct FeatureValue;
impl FeatureEntry for FeatureValue {
    type Output = f32;

    #[inline(always)]
    fn make_entry<F: Fn() -> String>(self, _: F, value: f32) -> Self::Output {
        value
    }
}

#[derive(Copy, Clone)]
pub struct FeatureUnit;
impl FeatureEntry for FeatureUnit {
    type Output = ();

    #[inline(always)]
    fn make_entry<F: Fn() -> String>(self, _: F, _: f32) -> Self::Output {}
}

#[repr(C)]
#[derive(Default, Copy, Clone, Serialize, Deserialize)]
pub struct TurnFeatures<T> {
    pub own_turn: T,
    pub opp_ended_round: T,
}

#[repr(C)]
#[derive(Default, Copy, Clone, Serialize, Deserialize)]
pub struct DiceFeatures<T> {
    pub on_count: T,
    pub off_count: T,
}

#[repr(C)]
#[derive(Default, Copy, Clone, Serialize, Deserialize)]
pub struct TeamStatusFeatures<T> {
    pub status_count: T,
    pub summon_count: T,
    pub support_count: T,
}

#[repr(C)]
#[derive(Default, Copy, Clone, Serialize, Deserialize)]
pub struct CharStatusFeatures<T> {
    pub equip_count: T,
    pub status_count: T,
}

#[repr(C)]
#[derive(Default, Copy, Clone, Serialize, Deserialize)]
pub struct CharFeatures<T> {
    pub is_active: T,
    pub is_alive: T,
    pub hp: T,
    pub energy: T,
    pub status: CharStatusFeatures<T>,
}

#[repr(C)]
#[derive(Default, Copy, Clone, Serialize, Deserialize)]
pub struct CanPerformFeatures<T> {
    pub switch: T,
    pub card: T,
    pub skill: T,
}

#[repr(C)]
#[derive(Default, Copy, Clone, Serialize, Deserialize)]
pub struct PlayerStateFeatures<T> {
    pub can_perform: CanPerformFeatures<T>,
    pub turn: TurnFeatures<T>,
    pub switch_is_fast_action: T,
    pub dice: DiceFeatures<T>,
    pub hand_count: T,
    pub team: TeamStatusFeatures<T>,
    pub chars: [CharFeatures<T>; CHAR_COUNT],
}

#[repr(C)]
#[derive(Default, Copy, Clone, Serialize, Deserialize)]
pub struct GameStateFeatures<T> {
    pub p1: PlayerStateFeatures<T>,
    pub p2: PlayerStateFeatures<T>,
}

impl_as_slice!(PlayerStateFeatures<f32>, f32);
impl_as_slice!(GameStateFeatures<f32>, f32);

trait BoolValue {
    fn bv(self) -> f32;
}

impl BoolValue for bool {
    #[inline(always)]
    fn bv(self) -> f32 {
        if self {
            1.0
        } else {
            0.0
        }
    }
}

impl PlayerState {
    fn char_status_features(&self, char_idx: u8) -> CharStatusFeatures<f32> {
        let sc = &self.status_collection;
        CharStatusFeatures {
            equip_count: sc.equipment_count(char_idx) as f32,
            status_count: sc.character_status_count(char_idx) as f32,
        }
    }

    fn char_features(&self, char_idx: u8) -> CharFeatures<f32> {
        if !self.is_valid_char_index(char_idx) {
            return Default::default();
        }

        let char_state = &self.char_states[char_idx as usize];
        CharFeatures {
            is_active: (self.active_char_index == char_idx).bv(),
            is_alive: true.bv(),
            hp: char_state.get_hp() as f32,
            energy: char_state.get_hp() as f32,
            status: self.char_status_features(char_idx),
        }
    }
    fn team_features(&self) -> TeamStatusFeatures<f32> {
        let sc = &self.status_collection;
        TeamStatusFeatures {
            status_count: sc.team_status_count() as f32,
            summon_count: sc.summon_count() as f32,
            support_count: sc.support_count() as f32,
        }
    }

    fn dice_features(&self) -> DiceFeatures<f32> {
        let dice_counter = &self.dice;
        let es = self.get_element_priority().elems();
        let off_count: u8 = Element::VALUES
            .iter()
            .copied()
            .filter(|&e| es.contains(e))
            .map(|e| dice_counter[Dice::Elem(e)])
            .sum();
        let on_count = dice_counter.total() - off_count;
        DiceFeatures {
            on_count: on_count as f32,
            off_count: off_count as f32,
        }
    }
}

impl GameState {
    fn can_perform_features(&self, player_id: PlayerId) -> CanPerformFeatures<f32> {
        if self.to_move_player() != Some(player_id) {
            return Default::default();
        }

        let actions = self.available_actions();
        CanPerformFeatures {
            switch: actions
                .iter()
                .any(|a| matches!(a, Input::FromPlayer(_, PlayerAction::SwitchCharacter(..))))
                .bv(),
            card: actions
                .iter()
                .any(|a| matches!(a, Input::FromPlayer(_, PlayerAction::PlayCard(..))))
                .bv(),
            skill: actions
                .iter()
                .any(|a| matches!(a, Input::FromPlayer(_, PlayerAction::CastSkill(..))))
                .bv(),
        }
    }

    fn player_state_features(&self, player_id: PlayerId) -> PlayerStateFeatures<f32> {
        let player_state = self.players.get(player_id);
        let switch_is_fast_action = (0u8..(CHAR_COUNT as u8))
            .into_iter()
            .any(|char_idx| self.check_switch_is_fast_action(player_id, char_idx))
            .bv();

        let turn = TurnFeatures {
            own_turn: (self.to_move_player() == Some(player_id)).bv(),
            opp_ended_round: self.phase.opponent_ended_round(player_id).bv(),
        };

        let mut chars: [CharFeatures<f32>; CHAR_COUNT] = Default::default();
        for (char_idx, c) in chars.iter_mut().enumerate() {
            *c = player_state.char_features(char_idx as u8);
        }

        PlayerStateFeatures {
            can_perform: self.can_perform_features(player_id),
            turn,
            switch_is_fast_action,
            dice: player_state.dice_features(),
            hand_count: player_state.hand.len() as f32,
            team: player_state.team_features(),
            chars,
        }
    }

    pub fn features(&self) -> GameStateFeatures<f32> {
        GameStateFeatures {
            p1: self.player_state_features(PlayerId::PlayerFirst),
            p2: self.player_state_features(PlayerId::PlayerSecond),
        }
    }

    pub fn features_vec<F: FeatureEntry<Output = V>, V>(&self, v: &mut Vec<V>, f: F) {
        const ENERGY: bool = true;
        const STATUSES: bool = true;
        const IS_ALIVE: bool = true;
        const IS_ACTIVE: bool = true;
        const COMPLEX_DICE: bool = false;
        const PER_ELEMENT: bool = false;
        macro_rules! entry {
            ($str: expr, $val: expr) => {
                f.make_entry((|| $str), $val as f32)
            };
        }

        for player_id in [PlayerId::PlayerFirst, PlayerId::PlayerSecond] {
            let player_state = &self.players[player_id];
            let status_collection = &player_state.status_collection;

            if COMPLEX_DICE {
                let d = DiceSummary::from_dice_counter(&player_state.dice, &player_state.get_element_priority());
                v.push(entry!(format!("{player_id}.Dice.OmniCount"), d.omni_count));
                v.push(entry!(format!("{player_id}.Dice.ActiveCount"), d.active_count));
                v.push(entry!(format!("{player_id}.Dice.ImportantCount"), d.important_count));
                v.push(entry!(format!("{player_id}.Dice.OffCount"), d.off_count));
            } else {
                v.push(entry!(format!("{player_id}.DiceCount"), player_state.dice.total()));
            }
            v.push(entry!(format!("{player_id}.HandCount"), player_state.hand.len() as f32));
            if STATUSES {
                let team_status_count = status_collection
                    .iter_entries()
                    .filter(|e| matches!(e.key, StatusKey::Team(..)))
                    .count();
                let summon_count = status_collection
                    .iter_entries()
                    .filter(|e| matches!(e.key, StatusKey::Summon(..)))
                    .count();
                let support_count = status_collection
                    .iter_entries()
                    .filter(|e| matches!(e.key, StatusKey::Support(..)))
                    .count();
                v.push(entry!(format!("{player_id}.TeamStatusCount"), team_status_count));
                v.push(entry!(format!("{player_id}.SummonCount"), summon_count));
                v.push(entry!(format!("{player_id}.SupportCount"), support_count));
            }

            let active_char_idx = player_state.active_char_index as usize;
            for (ci, char_state) in player_state.char_states.iter().enumerate() {
                let prefix = format!("{player_id}.{:?}", char_state.char_id);
                v.push(entry!(format!("{prefix}.HP"), char_state.get_hp() as f32));

                if IS_ALIVE {
                    v.push(entry!(
                        format!("{prefix}.IsAlive"),
                        if char_state.get_hp() > 0 { 1.0 } else { 0.0 }
                    ));
                }

                if ENERGY {
                    v.push(entry!(format!("{prefix}.Energy"), char_state.get_energy() as f32));
                }

                if IS_ACTIVE {
                    v.push(entry!(
                        format!("{prefix}.IsActive"),
                        if active_char_idx == ci { 1.0 } else { 0.0 }
                    ));
                }

                if STATUSES {
                    let equip_count = status_collection
                        .iter_entries()
                        .filter(|e| e.key.char_idx().is_some() && e.key.is_equipment())
                        .count();
                    let char_status_count = status_collection
                        .iter_entries()
                        .filter(|e| e.key.char_idx().is_some() && !e.key.is_equipment())
                        .count();
                    v.push(entry!(format!("{prefix}.EquipCount"), equip_count as f32));
                    v.push(entry!(format!("{prefix}.StatusCount"), char_status_count as f32));
                }

                let applied = char_state.applied;
                if PER_ELEMENT {
                    for e in Element::VALUES {
                        v.push(entry!(
                            format!("{prefix}.Elem.{e:?}"),
                            if applied.contains(e) { 1.0 } else { 0.0 }
                        ));
                    }
                } else {
                    v.push(entry!(format!("{prefix}.AppliedCount"), applied.len() as f32));
                }
            }
        }
    }
}

impl<S: NondetState> GameStateWrapper<S> {
    pub fn features_vec<F: FeatureEntry<Output = V>, V>(&self, v: &mut Vec<V>, f: F) {
        self.game_state.features_vec(v, f);
    }

    pub fn features_headers(&self) -> Vec<String> {
        let mut v = Vec::with_capacity(32);
        self.features_vec(&mut v, FeatureName);
        v
    }

    pub fn features(&self) -> Vec<f32> {
        let mut v = Vec::with_capacity(32);
        self.features_vec(&mut v, FeatureValue);
        v
    }

    pub fn features_len(&self) -> usize {
        let mut v = Vec::with_capacity(32);
        self.features_vec(&mut v, FeatureUnit);
        v.len()
    }
}

#[repr(C)]
#[derive(Default, Copy, Clone)]
pub struct PlayCardFeatures<T> {
    pub event_or_other: T,
    pub support: T,
    pub weapon_or_artifact: [T; N_CHARS],
}

#[repr(C)]
#[derive(Default, Copy, Clone)]
pub struct CastSkillFeatures<T> {
    pub normal_attack: T,
    pub elemental_skill: T,
    pub elemental_burst: T,
}

#[repr(C)]
#[derive(Default, Copy, Clone)]
pub struct InputFeatures<T> {
    pub end_round: T,
    pub switch: [T; N_CHARS],
    pub elemental_tuning: T,
    pub cast_skill: CastSkillFeatures<T>,
    pub play_card: PlayCardFeatures<T>,
}

impl_as_slice!(InputFeatures<f32>, f32);

macro_rules! vf {
    ($T: ident, $f: ident : $value: expr) => {
        $T {
            $f: $value,
            ..Default::default()
        }
    };
}

impl Input {
    pub fn features<T: Copy + Default>(self, value: T) -> InputFeatures<T> {
        match self {
            Input::NoAction => Default::default(),
            Input::NondetResult(_) => Default::default(),
            Input::FromPlayer(_, x) => match x {
                PlayerAction::EndRound => vf!(InputFeatures, end_round: value),
                PlayerAction::PlayCard(card_id, target) => Self::play_card_features(card_id, target, value),
                PlayerAction::ElementalTuning(..) => vf!(InputFeatures, elemental_tuning: value),
                PlayerAction::CastSkill(skill_id) => Self::cast_skill_features(skill_id, value),
                PlayerAction::SwitchCharacter(char_idx) | PlayerAction::PostDeathSwitch(char_idx) => InputFeatures {
                    switch: Self::from_char_idx(char_idx, value),
                    ..Default::default()
                },
            },
        }
    }

    fn play_card_features<T: Copy + Default>(
        card_id: CardId,
        target: Option<CardSelection>,
        value: T,
    ) -> InputFeatures<T> {
        let card_type = card_id.get_card().card_type;
        let play_card = match card_type {
            CardType::Event | CardType::Food | CardType::ElementalResonance(..) | CardType::Talent(..) => {
                vf!(PlayCardFeatures, event_or_other: value)
            }
            CardType::Support(..) => vf!(PlayCardFeatures, support: value),
            CardType::Weapon(..) | CardType::Artifact => {
                let targeting = target
                    .map(|t| match t {
                        CardSelection::OwnCharacter(char_idx) => Self::from_char_idx(char_idx, value),
                        CardSelection::OwnSummon(..) | CardSelection::OpponentSummon(..) => Default::default(),
                    })
                    .unwrap_or_default();
                PlayCardFeatures {
                    weapon_or_artifact: targeting,
                    ..Default::default()
                }
            }
        };
        InputFeatures {
            play_card,
            ..Default::default()
        }
    }

    fn cast_skill_features<T: Copy + Default>(skill_id: SkillId, value: T) -> InputFeatures<T> {
        let cast_skill = match skill_id.get_skill().skill_type {
            SkillType::NormalAttack => vf!(CastSkillFeatures, normal_attack: value),
            SkillType::ElementalSkill => vf!(CastSkillFeatures, elemental_skill: value),
            SkillType::ElementalBurst => vf!(CastSkillFeatures, elemental_burst: value),
        };
        InputFeatures {
            cast_skill,
            ..Default::default()
        }
    }

    fn from_char_idx<T: Copy + Default>(char_idx: u8, value: T) -> [T; N_CHARS] {
        let mut arr: [T; N_CHARS] = Default::default();
        arr[char_idx as usize] = value;
        arr
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use proptest::prelude::*;

    use crate::training::as_slice::AsSlice;

    type Slice = <InputFeatures<f32> as AsSlice>::Slice;
    proptest! {
        #[test]
        fn test_input_features_as_slice_roundtrip(slice in any::<Slice>()) {
            let input_features = InputFeatures::from_slice(slice);
            let slice1 = input_features.as_slice();
            assert_eq!(slice1, slice);
        }
    }
}
