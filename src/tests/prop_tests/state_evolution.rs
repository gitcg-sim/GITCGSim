use crate::types::game_state::PlayerId;

use super::*;

proptest! {
    #![proptest_config(ProptestConfig {
        cases: CASES,
        max_local_rejects: 2 * CASES,
        max_global_rejects: 2 * CASES,
        ..ProptestConfig::default()
    })]

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
}
