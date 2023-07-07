use super::*;

proptest! {
    #![proptest_config(ProptestConfig {
        cases: 10_000,
        max_local_rejects: 10_000,
        max_global_rejects: 10_000,
        ..ProptestConfig::default()
    })]

    #[test]
    fn test_game_state_serialize_bincode(gs in arb_reachable_game_state()) {
        let ser = bincode::serialize(&gs).unwrap();
        let mut gs1: GameState = bincode::deserialize(&ser).unwrap();
        gs1.rehash();
        assert_eq!(gs.zobrist_hash(), gs1.zobrist_hash());
    }

    #[test]
    fn test_game_state_wrapper_serialize_bincode(gs in arb_reachable_game_state_wrapper()) {
        let ser = bincode::serialize(&gs).unwrap();
        let mut gs1: GameStateWrapper<StandardNondetHandlerState> = bincode::deserialize(&ser).unwrap();
        gs1.game_state.rehash();
        assert_eq!(gs.zobrist_hash(), gs1.zobrist_hash());
    }

    #[test]
    fn test_game_state_serialize_json(gs in arb_reachable_game_state()) {
        let ser = serde_json::to_string_pretty(&gs).unwrap();
        let mut gs1: GameState = serde_json::from_str(&ser).unwrap();
        gs1.rehash();
        assert_eq!(gs.zobrist_hash(), gs1.zobrist_hash());
    }

    #[test]
    fn test_game_state_wrapper_serialize_json(gs in arb_reachable_game_state_wrapper()) {
        let ser = serde_json::to_string_pretty(&gs).unwrap();
        let mut gs1: GameStateWrapper<StandardNondetHandlerState> = serde_json::from_str(&ser).unwrap();
        gs1.game_state.rehash();
        assert_eq!(gs.zobrist_hash(), gs1.zobrist_hash());
    }
}
