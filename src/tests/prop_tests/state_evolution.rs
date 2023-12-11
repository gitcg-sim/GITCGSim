use crate::list8;

use super::*;

proptest! {
    #![proptest_config(ProptestConfig {
        cases: CASES,
        max_local_rejects: 2 * CASES,
        max_global_rejects: 2 * CASES,
        ..ProptestConfig::default()
    })]

    #[test]
    fn select_starting_character(gs in arb_init_game_state(), a in 0u8..=2, b in 0u8..=2) {
        let a = a % gs.get_player(PlayerId::PlayerFirst).char_states.len();
        let b = b % gs.get_player(PlayerId::PlayerSecond).char_states.len();
        let mut gs = gs;
        gs.advance(Input::FromPlayer(PlayerId::PlayerFirst, PlayerAction::SwitchCharacter(a))).unwrap();
        gs.advance(Input::FromPlayer(PlayerId::PlayerSecond, PlayerAction::SwitchCharacter(b))).unwrap();
        assert_eq!(Phase::new_roll_phase(PlayerId::PlayerFirst), gs.phase);
        assert_eq!(a, gs.get_player(PlayerId::PlayerFirst).active_char_idx);
        assert_eq!(b, gs.get_player(PlayerId::PlayerSecond).active_char_idx);
        gs.advance(Input::NoAction).unwrap();
        gs.advance(Input::NondetResult(NondetResult::ProvideCards(list8![], list8![]))).unwrap();
        gs.advance(Input::NondetResult(NondetResult::ProvideDice(Default::default(), Default::default()))).unwrap();

        assert_eq!(Some(PlayerId::PlayerFirst), gs.to_move_player());
        assert_eq!(a, gs.get_player(PlayerId::PlayerFirst).active_char_idx);
        assert_eq!(b, gs.get_player(PlayerId::PlayerSecond).active_char_idx);
    }

    #[test]
    fn actions_from_actions_should_be_performable_with_ok_result(gs in arb_reachable_game_state_wrapper()) {
        for action in gs.actions() {
            let mut gs1 = gs.clone();
            if let Err(e) = gs1.advance(action) {
                dbg!(&gs);
                dbg!(&action);
                dbg!(&e);
                panic!("Action is not performable.");
            }
        }
    }

    #[test]
    fn cannot_contain_blank_card_on_hand(gs in arb_reachable_game_state()) {
        assert!(!gs.get_player(crate::types::game_state::PlayerId::PlayerFirst).hand.contains(&CardId::BlankCard));
        assert!(!gs.get_player(crate::types::game_state::PlayerId::PlayerSecond).hand.contains(&CardId::BlankCard));
    }

    #[test]
    fn can_perform_available_action_for_non_winner_game_states(gs in arb_reachable_game_state(), action in arb_player_action(), player_id in arb_player_id()) {
        prop_assume!(gs.phase.winner().is_none(), "must not have a winner");
        let input = Input::FromPlayer(player_id, action);
        let aa = gs.available_actions();
        let mut gs1 = gs;
        assert_eq!(aa.contains(&input), gs1.advance(input).is_ok())
    }

    #[test]
    fn returns_winner_decided_for_winner_game_states(gs in arb_reachable_game_state_winner(), action in arb_player_action(), player_id in arb_player_id()) {
        let gs = gs.game_state;
        prop_assume!(gs.phase.winner().is_some(), "must have a winner");
        let winner = gs.phase.winner().unwrap();
        let input = Input::FromPlayer(player_id, action);
        let mut gs1 = gs;
        assert_eq!(Ok(DispatchResult::Winner(winner)), gs1.advance(input))
    }

    #[test]
    fn status_collections_are_sorted_by_sort_key(gs in arb_reachable_game_state_wrapper()) {
        fn is_sorted<T: Ord>(v: &Vec<T>) -> bool {
            if v.len() <= 1 {
                return true
            }
            for i in 0..=v.len() - 2 {
                if v[i] > v[i + 1] {
                    return false
                }
            }
            true
        }

        let gs = gs.game_state;
        for sc in [&gs.get_player(PlayerId::PlayerFirst).status_collection, &gs.get_player(PlayerId::PlayerSecond).status_collection ] {
            let sort_keys: Vec<_> = sc._status_entries.iter().map(|s| s.key.sort_key()).collect();
            assert!(is_sorted(&sort_keys));
        }
    }

    #[test]
    fn hide_private_information_for_opposite_player_preserves_own_dice_hand_flags(gs in arb_reachable_game_state_wrapper()) {
        prop_assume!(gs.winner().is_none(), "must not have a winner");
        let player_id = gs.to_move().unwrap();
        let mut gs1 = gs;
        let (dice1, hand1, flags1) = {
            let p = &gs1.game_state.get_player(player_id);
            (p.dice, p.hand.clone(), p.flags)
        };
        gs1.hide_private_information(player_id.opposite());
        let (dice2, hand2, flags2) = {
            let p = &gs1.game_state.get_player(player_id);
            (p.dice, p.hand.clone(), p.flags)
        };
        assert_eq!(dice1, dice2);
        assert_eq!(hand1, hand2);
        assert_eq!(flags1, flags2);
    }

    #[test]
    fn hide_private_information_for_opposite_player_preserves_own_available_actions(gs in arb_reachable_game_state_wrapper()) {
        prop_assume!(gs.winner().is_none(), "must not have a winner");
        let player_id = gs.to_move().unwrap();
        let aa = gs.actions();
        let mut gs1 = gs;
        gs1.hide_private_information(player_id.opposite());
        let aa1 = gs1.actions();
        assert_eq!(aa, aa1);
    }

    #[test]
    fn non_active_characters_no_plunging_attack_flag(gs in arb_reachable_game_state_wrapper()) {
        for player_id in [PlayerId::PlayerFirst, PlayerId::PlayerSecond] {
            let PlayerState { char_states, active_char_idx, .. } = gs.game_state.players.get(player_id);
            for (char_idx, char_state) in char_states.enumerate_valid().filter(|(i, _)| i != active_char_idx) {
                assert!(
                    !char_state.flags.contains(crate::types::char_state::CharFlag::PlungingAttack),
                    "Contains PlungingAttack: {char_state:?}, player_id={player_id}, active_char_idx={active_char_idx}, char_idx={char_idx}"
                );
            }
        }
    }

    #[test]
    fn test_rewritten_arb_game_state(gs in
        ArbGameState::new(
            arb_decklist_with_chars(arb_char_ids_containing(CharId::Beidou)),
            arb_decklist_with_chars(arb_char_ids_containing(CharId::Wanderer))
        ).arb_reachable().arb()
    ) {
        assert!(gs.game_state.get_player(PlayerId::PlayerFirst).char_states.iter_all().any(|c| c.char_id == CharId::Beidou));
        assert!(gs.game_state.get_player(PlayerId::PlayerSecond).char_states.iter_all().any(|c| c.char_id == CharId::Wanderer));
    }
}
