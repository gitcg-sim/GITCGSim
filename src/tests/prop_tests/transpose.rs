use super::*;

use crate::data_structures::ActionList;

fn transpose_actions(xs: ActionList<Input>) -> ActionList<Input> {
    xs.iter().copied().map(Input::transpose).collect()
}

fn advance<P: GameStateParams>(game_state: &GameState<P>, input: Input) -> GameState<P> {
    let mut gs1 = game_state.clone();
    gs1.advance(input).unwrap();
    gs1
}

fn action<P: GameStateParams>(gs: &GameState<P>, n: usize) -> Result<Input, TestCaseError> {
    let actions = gs.available_actions();
    prop_assume!(!actions.is_empty());
    Ok(actions[n % actions.len()])
}

// t(G), t(a): transpose [game state G]/[action a]
// advance(G, a): advance game state G with action a
proptest! {
    #![proptest_config(ProptestConfig {
        cases: CASES,
        max_local_rejects: 2 * CASES,
        max_global_rejects: 2 * CASES,
        ..ProptestConfig::default()
    })]

    /// `t(t(G)) = G`
    #[test]
    fn double_transpose_equals_itself(gs in arb_reachable_game_state()) {
        let gs1 = gs.transpose().transpose();
        assert_eq!(format!("{gs:?}"), format!("{gs1:?}"));
        assert_eq!(gs.zobrist_hash(), gs1.zobrist_hash());
    }

    /// `actions(t(G)) = [ t(a) | a <- actions(G) ]`
    #[test]
    fn transpose_symmetry_for_available_actions_left(gs in arb_reachable_game_state()) {
        let gs1 = gs.transpose();
        let actions = gs1.available_actions();
        let actions1 = transpose_actions(gs.available_actions());
        assert_eq!(actions, actions1);
    }

    /// `actions(G) = [ t(a) | a <- actions(t(G)) ]`
    #[test]
    fn transpose_symmetry_for_available_actions_right(gs in arb_reachable_game_state()) {
        let actions = gs.available_actions();
        let gs1 = gs.transpose();
        let actions1 = transpose_actions(gs1.available_actions());
        assert_eq!(actions, actions1);
    }

    /// `advance(G, a) = t(advance(t(G), t(a))) where a in actions(G)`
    #[test]
    fn advance_under_transpose(gs in arb_reachable_game_state(), n in any::<usize>()) {
        let a = action(&gs, n)?;
        let gs1 = advance(&gs, a);
        let gs2 = advance(&gs.transpose(), a.transpose()).transpose();
        assert_eq!(format!("{gs1:?}"), format!("{gs2:?}"));
        assert_eq!(gs1.zobrist_hash(), gs2.zobrist_hash());
    }

    /// `advance(advance(G, a), a') = t(advance(advance(t(G), t(a)), t(a'))) where a in actions(G), a' in actions(advance(G, a))`
    #[test]
    fn advance_under_transpose_secondary(gs in arb_reachable_game_state(), n in any::<usize>(), m in any::<usize>()) {
        let a = action(&gs, n)?;
        let gs1 = advance(&gs, a);
        let a1 = action(&gs1, m)?;
        let gs1 = advance(&gs1, a1);
        let gs2 = advance(&advance(&gs.transpose(), a.transpose()), a1.transpose()).transpose();
        assert_eq!(format!("{gs1:?}"), format!("{gs2:?}"));
        assert_eq!(gs1.zobrist_hash(), gs2.zobrist_hash());
    }
}
