use crate::cards::ids::{CardId, GetCard, GetSkill};
use crate::impl_as_slice;
use crate::prelude::Input;
use crate::status_impls::prelude::*;
use crate::types::card_defs::CardType;
use crate::types::input::PlayerAction;
use crate::{
    game_tree_search::GameStateWrapper,
    types::{
        dice_counter::{DiceCounter, ElementPriority},
        game_state::*,
        nondet::NondetState,
    },
};

pub const N_CHARS: usize = 3;

struct DiceFeatures {
    pub omni_count: u8,
    pub active_count: u8,
    pub important_count: u8,
    pub off_count: u8,
}

impl DiceFeatures {
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

impl GameState {
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
                let d = DiceFeatures::from_dice_counter(&player_state.dice, &player_state.get_element_priority());
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
