use crate::cards::ids::{CardId, GetCard, GetSkill};
use crate::impl_as_slice;
use crate::prelude::*;
use crate::status_impls::prelude::*;
use crate::types::card_defs::CardType;
use crate::types::game_state::*;
use crate::types::input::PlayerAction;
use crate::types::nondet::NondetState;

/// Number of characters to be included in features.
pub const N_CHARS: usize = 3;

#[repr(C)]
#[derive(Default, Copy, Clone)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct TurnFeatures<T> {
    pub own_turn: T,
    pub opp_ended_round: T,
}

#[repr(C)]
#[derive(Default, Copy, Clone)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct DiceFeatures<T> {
    pub on_count: T,
    pub off_count: T,
}

#[repr(C)]
#[derive(Default, Copy, Clone)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct TeamStatusFeatures<T> {
    pub status_count: T,
    pub summon_count: T,
    pub support_count: T,
}

#[repr(C)]
#[derive(Default, Copy, Clone)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct CharStatusFeatures<T> {
    pub equip_count: T,
    pub status_count: T,
}

#[repr(C)]
#[derive(Default, Copy, Clone)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct CharFeatures<T> {
    pub is_active: T,
    pub is_alive: T,
    pub hp: T,
    pub energy: T,
    pub applied_count: T,
    pub status: CharStatusFeatures<T>,
}

#[repr(C)]
#[derive(Default, Copy, Clone)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct CanPerformFeatures<T> {
    pub switch: T,
    pub card: T,
    pub skill: T,
}

#[repr(C)]
#[derive(Default, Copy, Clone)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct PlayerStateFeatures<T> {
    pub can_perform: CanPerformFeatures<T>,
    pub turn: TurnFeatures<T>,
    pub switch_is_fast_action: T,
    pub dice: DiceFeatures<T>,
    pub hand_count: T,
    pub team: TeamStatusFeatures<T>,
    pub chars: [CharFeatures<T>; N_CHARS],
}

#[repr(C)]
#[derive(Default, Copy, Clone)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
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
            energy: char_state.get_energy() as f32,
            applied_count: char_state.applied.len() as f32,
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
            .filter(|&e| !es.contains(e))
            .map(|e| dice_counter[Dice::Elem(e)])
            .sum();
        debug_assert!(off_count <= dice_counter.total());
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
        let switch_is_fast_action = (0u8..(N_CHARS as u8))
            .any(|char_idx| self.check_switch_is_fast_action(player_id, char_idx))
            .bv();

        let turn = TurnFeatures {
            own_turn: (self.to_move_player() == Some(player_id)).bv(),
            opp_ended_round: self.phase.opponent_ended_round(player_id).bv(),
        };

        let mut chars: [CharFeatures<f32>; N_CHARS] = Default::default();
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
}

impl<S: NondetState> crate::game_tree_search::GameStateWrapper<S> {
    pub fn features(&self) -> GameStateFeatures<f32> {
        self.game_state.features()
    }
}

#[repr(C)]
#[derive(Default, Copy, Clone)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct PlayCardFeatures<T> {
    pub event_or_other: T,
    pub support: T,
    pub weapon_or_artifact: [T; N_CHARS],
}

#[repr(C)]
#[derive(Default, Copy, Clone)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct CastSkillFeatures<T> {
    pub normal_attack: T,
    pub elemental_skill: T,
    pub elemental_burst: T,
}

#[repr(C)]
#[derive(Default, Copy, Clone)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct InputFeatures<T> {
    pub end_round: T,
    pub switch: [T; N_CHARS],
    pub elemental_tuning: T,
    pub cast_skill: CastSkillFeatures<T>,
    pub play_card: PlayCardFeatures<T>,
}

impl_as_slice!(InputFeatures<f32>, f32);

impl_as_slice!(InputFeatures<GameStateFeatures<f32>>, f32);

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

    type Slice = <InputFeatures<f32> as AsSlice<f32>>::Slice;

    proptest! {
        #[test]
        fn test_input_features_as_slice_roundtrip(slice in any::<Slice>()) {
            let input_features = <InputFeatures<f32> as AsSlice<f32>>::from_slice(slice);
            let slice1 = input_features.as_slice();
            assert_eq!(slice1, slice);
        }
    }
}
