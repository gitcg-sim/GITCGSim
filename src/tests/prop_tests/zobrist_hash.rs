use crate::cards::ids::CharId;

use super::*;

proptest! {
    #![proptest_config(ProptestConfig {
        cases: 10_000,
        max_local_rejects: 100_000,
        max_global_rejects: 100_000,
        ..ProptestConfig::default()
    })]

    #[test]
    fn actions_should_preserve_incremental_hash_for_game_state_wrapper((gs, action) in arb_reachable_game_state_wrapper_with_action()) {
        let h0 = ZobristHashable::zobrist_hash(&gs);
        let h_incremental = {
            let mut gs1 = gs.clone();
            gs1.advance(action).unwrap();
            ZobristHashable::zobrist_hash(&gs1)
        };
        let h_rehash = {
            let mut gs1 = gs;
            gs1.advance(action).unwrap();
            gs1.game_state.rehash();
            ZobristHashable::zobrist_hash(&gs1)
        };
        assert_eq!(h_rehash, h_incremental, "Init = {h0:?}, {action:?}");
    }

    #[test]
    fn actions_should_preserve_incremental_hash_for_initial_game_state((gs, action) in arb_init_game_state_wrapper_with_action()) {
        let h0 = gs.game_state.zobrist_hash();
        let h_incremental = {
            let mut gs1 = gs.clone();
            gs1.advance(action).unwrap();
            gs1.game_state.zobrist_hash()
        };
        let h_rehash = {
            let mut gs1 = gs;
            gs1.advance(action).unwrap();
            gs1.game_state.rehash();
            gs1.game_state.zobrist_hash()
        };
        assert_eq!(h_rehash, h_incremental, "Init = {h0:?}, action = {action:?}");
    }

    #[test]
    fn actions_should_preserve_incremental_hash_for_game_state((gs, action) in arb_reachable_game_state_wrapper_with_action()) {
        let h0 = gs.game_state.zobrist_hash();
        let h_incremental = {
            let mut gs1 = gs.clone();
            gs1.advance(action).unwrap();
            gs1.game_state.zobrist_hash()
        };
        let h_rehash = {
            let mut gs1 = gs;
            gs1.advance(action).unwrap();
            gs1.game_state.rehash();
            gs1.game_state.zobrist_hash()
        };
        assert_eq!(h_rehash, h_incremental, "Init = {h0:?}, {action:?}");
    }

    #[test]
    fn actions_should_preserve_incremental_hash_for_game_state_prepared_skill((gs, action) in arb_reachable_game_state_wrapper_with_action()) {
        prop_assume!(gs.game_state.players.0.char_states.iter().any(|c| c.char_id == CharId::Beidou || c.char_id == CharId::Candace));
        let h0 = gs.game_state.zobrist_hash();
        let h_incremental = {
            let mut gs1 = gs.clone();
            gs1.advance(action).unwrap();
            gs1.game_state.zobrist_hash()
        };
        let h_rehash = {
            let mut gs1 = gs;
            gs1.advance(action).unwrap();
            gs1.game_state.rehash();
            gs1.game_state.zobrist_hash()
        };
        assert_eq!(h_rehash, h_incremental, "Init = {h0:?}, {action:?}");
    }
}
