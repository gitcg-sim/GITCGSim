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

pub trait SelectionPolicy<G: Game>: Send + Sync {
    type State;

    /// Utility function for calculating the Cpuct
    fn cpuct(ctx: &SelectionPolicyContext<G>, parent_n: u32) -> f32 {
        ctx.config.cpuct.cpuct(parent_n as f32)
    }

    fn uct_parent_factor<F: FnOnce() -> G::Actions>(
        &self,
        ctx: &SelectionPolicyContext<G>,
        get_children: F,
    ) -> Self::State;
    fn uct_child_factor(
        &self,
        ctx: &SelectionPolicyContext<G>,
        index: usize,
        child: &NodeData<G>,
        state: &Self::State,
    ) -> f32;
}

#[derive(Default, Copy, Clone)]
pub struct NoneUCT;
impl<G: Game> SelectionPolicy<G> for NoneUCT {
    type State = ();

    fn uct_parent_factor<F: FnOnce() -> G::Actions>(&self, _: &SelectionPolicyContext<G>, _: F) {}

    fn uct_child_factor(&self, _: &SelectionPolicyContext<G>, _: usize, _: &NodeData<G>, _: &Self::State) -> f32 {
        0f32
    }
}

#[derive(Default, Copy, Clone)]
pub struct UCB1;

impl<G: Game> SelectionPolicy<G> for UCB1 {
    type State = f32;

    fn uct_parent_factor<F: FnOnce() -> G::Actions>(&self, ctx: &SelectionPolicyContext<G>, _: F) -> f32 {
        let n_parent = ctx.parent.prop.n;
        ctx.config.cpuct.init * (n_parent as f32).ln_1p()
    }

    fn uct_child_factor(&self, _: &SelectionPolicyContext<G>, _: usize, child: &NodeData<G>, factor: &f32) -> f32 {
        let n_child = child.prop.n + 1;
        (factor / (n_child as f32)).sqrt()
    }
}
