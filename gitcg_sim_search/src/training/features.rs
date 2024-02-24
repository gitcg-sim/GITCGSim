use std::ops::Add;

use crate::impl_as_slice;
use gitcg_sim::{
    enum_map::Enum,
    prelude::{card_defs::*, tcg_model::*, *},
};

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

impl<T: Copy + Add<Output = T>> TeamStatusFeatures<T> {
    #[inline(always)]
    fn total(&self) -> T {
        self.status_count + self.summon_count + self.support_count
    }
}

#[repr(C)]
#[derive(Default, Copy, Clone)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct CharStatusFeatures<T> {
    pub equip_count: T,
    pub status_count: T,
}

impl<T: Copy + Add<Output = T>> CharStatusFeatures<T> {
    #[inline(always)]
    fn total(&self) -> T {
        self.equip_count + self.status_count
    }
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

#[repr(C)]
#[derive(Default, Copy, Clone)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct ExpressCharFeatures<T> {
    pub has_applied: T,
    pub hp: T,
    pub energy: T,
    // pub status: CharStatusFeatures<T>,
    pub status_count: T,
}

#[repr(C)]
#[derive(Default, Copy, Clone)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct ExpressPlayerStateFeatures<T> {
    // pub can_perform: CanPerformFeatures<T>,
    pub turn: TurnFeatures<T>,
    pub dice: DiceFeatures<T>,
    pub hand_count: T,
    // pub team: TeamStatusFeatures<T>,
    pub team_status_count: T,
    pub active_char: ExpressCharFeatures<T>,
    pub inactive_chars: [ExpressCharFeatures<T>; N_CHARS],
    // serde-compatible way for [T; 128]
    pub char_ids: [[[T; 32]; 4]; N_CHARS],
}

#[repr(C)]
#[derive(Default, Copy, Clone)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct ExpressGameStateFeatures<T> {
    pub p1: ExpressPlayerStateFeatures<T>,
    pub p2: ExpressPlayerStateFeatures<T>,
}

impl_as_slice!(ExpressCharFeatures<f32>, f32);
impl_as_slice!(ExpressPlayerStateFeatures<f32>, f32);
impl_as_slice!(ExpressGameStateFeatures<f32>, f32);

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

pub mod player_state_features {
    use super::*;

    fn char_status_features(player_state: &PlayerState, char_idx: u8) -> CharStatusFeatures<f32> {
        let sc = player_state.get_status_collection();
        CharStatusFeatures {
            equip_count: sc.equipment_count(char_idx) as f32,
            status_count: sc.character_status_count(char_idx) as f32,
        }
    }

    pub fn char_features(player_state: &PlayerState, char_idx: u8) -> CharFeatures<f32> {
        if !player_state.is_valid_char_idx(char_idx) {
            return Default::default();
        }

        let char_state = &player_state.char_states[char_idx];
        CharFeatures {
            is_active: (player_state.get_active_char_idx() == char_idx).bv(),
            is_alive: true.bv(),
            hp: char_state.get_hp() as f32,
            energy: char_state.get_energy() as f32,
            applied_count: char_state.applied.len() as f32,
            status: char_status_features(player_state, char_idx),
        }
    }

    pub fn express_char_features(player_state: &PlayerState, char_idx: u8) -> ExpressCharFeatures<f32> {
        if !player_state.is_valid_char_idx(char_idx) {
            return Default::default();
        }

        let char_state = &player_state.char_states[char_idx];
        ExpressCharFeatures {
            has_applied: if char_state.applied.is_empty() { 0.0 } else { 1.0 },
            hp: char_state.get_hp() as f32,
            energy: char_state.get_energy() as f32,
            status_count: char_status_features(player_state, char_idx).total(),
        }
    }

    pub fn team_features(player_state: &PlayerState) -> TeamStatusFeatures<f32> {
        let sc = player_state.get_status_collection();
        TeamStatusFeatures {
            status_count: sc.team_status_count() as f32,
            summon_count: sc.summon_count() as f32,
            support_count: sc.support_count() as f32,
        }
    }

    pub fn dice_features(player_state: &PlayerState) -> DiceFeatures<f32> {
        let dice_counter = player_state.get_dice_counter();
        let es = player_state.get_element_priority().elems();
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

pub mod game_state_features {
    use super::*;
    use player_state_features::*;

    fn can_perform_features(game_state: &GameState, player_id: PlayerId) -> CanPerformFeatures<f32> {
        if game_state.to_move_player() != Some(player_id) {
            return Default::default();
        }

        let actions = game_state.available_actions();
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

    fn turn_features(game_state: &GameState, player_id: PlayerId) -> TurnFeatures<f32> {
        TurnFeatures {
            own_turn: (game_state.to_move_player() == Some(player_id)).bv(),
            opp_ended_round: game_state.get_phase().opponent_ended_round(player_id).bv(),
        }
    }

    fn express_player_state_features(game_state: &GameState, player_id: PlayerId) -> ExpressPlayerStateFeatures<f32> {
        let player_state = game_state.get_player(player_id);
        let active_char_idx = player_state.get_active_char_idx();
        let mut chars: [ExpressCharFeatures<f32>; N_CHARS] = Default::default();
        for (char_idx, c) in chars.iter_mut().enumerate() {
            *c = express_char_features(player_state, char_idx as u8);
        }

        let mut active_char = Default::default();
        std::mem::swap(&mut active_char, &mut chars[active_char_idx as usize]);

        let mut char_ids = [[[0.0; 32]; 4]; N_CHARS];
        for (i, n) in player_state
            .char_states
            .iter_all()
            .map(|s| s.char_id.into_usize())
            .enumerate()
        {
            char_ids[i][n / 32][n % 32] = 1.0;
        }

        ExpressPlayerStateFeatures {
            // can_perform: self.can_perform_features(player_id),
            turn: turn_features(game_state, player_id),
            dice: dice_features(player_state),
            hand_count: player_state.hand_len() as f32,
            // team: player_state.team_features(),
            team_status_count: team_features(player_state).total(),
            active_char,
            inactive_chars: chars,
            char_ids,
        }
    }

    fn player_state_features(game_state: &GameState, player_id: PlayerId) -> PlayerStateFeatures<f32> {
        let player_state = game_state.get_player(player_id);
        let switch_is_fast_action = (0u8..(N_CHARS as u8))
            .any(|char_idx| game_state.check_switch_is_fast_action(player_id, char_idx))
            .bv();

        let mut chars: [CharFeatures<f32>; N_CHARS] = Default::default();
        for (char_idx, c) in chars.iter_mut().enumerate() {
            *c = char_features(player_state, char_idx as u8);
        }

        PlayerStateFeatures {
            can_perform: can_perform_features(game_state, player_id),
            turn: turn_features(game_state, player_id),
            switch_is_fast_action,
            dice: dice_features(player_state),
            hand_count: player_state.hand.len() as f32,
            team: team_features(player_state),
            chars,
        }
    }

    // Unused
    pub fn extended_features(game_state: &GameState) -> GameStateFeatures<f32> {
        GameStateFeatures {
            p1: player_state_features(game_state, PlayerId::PlayerFirst),
            p2: player_state_features(game_state, PlayerId::PlayerSecond),
        }
    }

    pub fn features(game_state: &GameState) -> ExpressGameStateFeatures<f32> {
        ExpressGameStateFeatures {
            p1: express_player_state_features(game_state, PlayerId::PlayerFirst),
            p2: express_player_state_features(game_state, PlayerId::PlayerSecond),
        }
    }
}

// impl<S: NondetState> gitcg_sim::game_tree_search::GameStateWrapper<S> {
//     #[cfg(any())]
//     pub fn features(&self) -> GameStateFeatures<f32> {
//         self.game_state.features()
//     }
//
//     pub fn features(&self) -> ExpressGameStateFeatures<f32> {
//         self.game_state.features()
//     }
// }

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

/// Struct for the feature vector describing a player action.
/// Invariant: For any given player action, only ONE field of the features is non-zero.
/// TODO write a property test on this invariant
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

pub mod input_features {
    use super::*;

    pub fn input_features<T: Copy + Default>(input: Input, value: T) -> InputFeatures<T> {
        match input {
            Input::NoAction => Default::default(),
            Input::NondetResult(_) => Default::default(),
            Input::FromPlayer(_, x) => match x {
                PlayerAction::EndRound => vf!(InputFeatures, end_round: value),
                PlayerAction::PlayCard(card_id, target) => play_card_features(card_id, target, value),
                PlayerAction::ElementalTuning(..) => vf!(InputFeatures, elemental_tuning: value),
                PlayerAction::CastSkill(skill_id) => cast_skill_features(skill_id, value),
                PlayerAction::SwitchCharacter(char_idx) | PlayerAction::PostDeathSwitch(char_idx) => vf!(
                    InputFeatures,
                    switch: from_char_idx(char_idx, value)
                ),
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
                        CardSelection::OwnCharacter(char_idx) => from_char_idx(char_idx, value),
                        CardSelection::OwnSummon(..) | CardSelection::OpponentSummon(..) => Default::default(),
                    })
                    .unwrap_or_default();
                vf!(PlayCardFeatures, weapon_or_artifact: targeting)
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

pub type Features = ExpressGameStateFeatures<f32>;

#[cfg(test)]
mod test {
    use super::*;
    use crate::training::as_slice::AsSlice;
    use proptest::prelude::*;

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
