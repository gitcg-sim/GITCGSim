use crate::training::policy::SelectionPolicyState;
use gitcg_sim::prelude::NondetState;

use super::*;

pub trait EvalPolicy<G: Game>: Send + Sync {
    fn evaluate(&self, state: &G, player_id: PlayerId) -> G::Eval;
}

#[derive(Default)]
pub struct DefaultEvalPolicy();

impl<G: Game> EvalPolicy<G> for DefaultEvalPolicy {
    #[inline(always)]
    fn evaluate(&self, state: &G, player_id: PlayerId) -> G::Eval {
        state.eval(player_id)
    }
}

pub struct SelectionPolicyContext<'a, 'b, G: Game> {
    pub config: &'a MCTSConfig,
    pub parent: &'b NodeData<G>,
    pub is_maximize: bool,
}

pub struct SelectionPolicyChildContext<'a, 'b, G: Game, S> {
    pub index: usize,
    pub child: &'a NodeData<G>,
    pub state: &'b S,
}

/// Trait for customizing various aspects of MCTS selection policy.
/// In the selection phase of MCTS, the node with highest (ratio + uct)
/// is selected, where ratio is the MCTS win rate of the particular node
/// and uct is a factor to encourage nodes with fewer visits to be selected.
///
/// See also: <https://en.wikipedia.org/wiki/Monte_Carlo_tree_search#Exploration_and_exploitation>
pub trait SelectionPolicy<G: Game>: Send + Sync {
    /// The state stored while evaluting the parent node and can be used
    /// by the child nodes.
    type State;

    /// Utility function for calculating the Cpuct. Do not override.
    fn cpuct(ctx: &SelectionPolicyContext<G>, parent_n: u32) -> f32 {
        ctx.config.cpuct.cpuct(parent_n as f32)
    }

    /// Evaluate the parent node.
    fn on_parent<F: FnOnce() -> G::Actions>(&self, ctx: &SelectionPolicyContext<G>, get_children: F) -> Self::State;

    /// Evaluate the policy value of a particular child node.
    fn policy(&self, _ctx: &SelectionPolicyContext<G>, _cctx: &SelectionPolicyChildContext<G, Self::State>) -> f32 {
        1.0
    }

    /// Evaluate the UCT value of a particular child given policy value.
    fn uct_child(
        &self,
        ctx: &SelectionPolicyContext<G>,
        cctx: &SelectionPolicyChildContext<G, Self::State>,
        policy_value: f32,
    ) -> f32;
}

#[derive(Default, Copy, Clone)]
pub struct NoneUCT;
impl<G: Game> SelectionPolicy<G> for NoneUCT {
    type State = ();

    fn on_parent<F: FnOnce() -> G::Actions>(&self, _: &SelectionPolicyContext<G>, _: F) {}

    fn uct_child(&self, _: &SelectionPolicyContext<G>, _: &SelectionPolicyChildContext<G, Self::State>, _: f32) -> f32 {
        0f32
    }
}

#[derive(Default, Copy, Clone)]
pub struct UCB1;

impl<G: Game> SelectionPolicy<G> for UCB1 {
    type State = f32;

    fn on_parent<F: FnOnce() -> G::Actions>(&self, ctx: &SelectionPolicyContext<G>, _: F) -> f32 {
        let n_parent = ctx.parent.prop.n;
        ctx.config.cpuct.init * (n_parent as f32).ln_1p()
    }

    fn uct_child(
        &self,
        _: &SelectionPolicyContext<G>,
        cctx: &SelectionPolicyChildContext<G, Self::State>,
        _: f32,
    ) -> f32 {
        let factor = *cctx.state;
        let n_child = cctx.child.prop.n + 1;
        (factor / (n_child as f32)).sqrt()
    }
}

#[derive(Default, Copy, Clone)]
pub struct RuleBasedPuct;

impl<S: NondetState> SelectionPolicy<GameStateWrapper<S>> for RuleBasedPuct {
    type State = SelectionPolicyState;

    fn on_parent<F: FnOnce() -> <GameStateWrapper<S> as Game>::Actions>(
        &self,
        ctx: &SelectionPolicyContext<GameStateWrapper<S>>,
        get_children: F,
    ) -> Self::State {
        let parent = ctx.parent;
        let mut gs = parent.state.game_state.clone();
        if !ctx.is_maximize {
            gs.transpose_in_place();
        }
        let actions = get_children();
        let evals = parent
            .state
            .action_weights(&actions)
            .iter()
            .copied()
            .map(|(_, x)| ctx.config.policy_softmax(x))
            .collect::<SmallVec<_>>();
        let denominator = 1e-5 + evals.iter().sum::<f32>();
        let n = parent.prop.n + 1;
        SelectionPolicyState {
            puct_mult: Self::cpuct(ctx, parent.prop.n) * (n as f32).sqrt(),
            evals,
            denominator,
        }
    }

    fn policy(
        &self,
        _: &SelectionPolicyContext<GameStateWrapper<S>>,
        cctx: &SelectionPolicyChildContext<GameStateWrapper<S>, Self::State>,
    ) -> f32 {
        cctx.state.evals[cctx.index] / cctx.state.denominator
    }

    fn uct_child(
        &self,
        _: &SelectionPolicyContext<GameStateWrapper<S>>,
        cctx: &SelectionPolicyChildContext<GameStateWrapper<S>, Self::State>,
        policy_value: f32,
    ) -> f32 {
        let state = &cctx.state;
        let puct_mult = state.puct_mult;
        let n = cctx.child.prop.n;
        let fpu = if n < 1 { 1.0 } else { 0.0 };
        policy_value * puct_mult / ((n + 1) as f32) + fpu
    }
}
