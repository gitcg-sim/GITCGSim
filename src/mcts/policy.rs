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
    pub parent: &'b Node<G>,
    pub is_maximize: bool,
}

pub trait SelectionPolicy<G: Game>: Send + Sync {
    type State;
    fn uct_parent_factor(&self, ctx: &SelectionPolicyContext<G>) -> Self::State;
    fn uct_child_factor(&self, ctx: &SelectionPolicyContext<G>, child: &Node<G>, state: &Self::State) -> f32;
}

#[derive(Default, Copy, Clone)]
pub struct NoneUCT;
impl<G: Game> SelectionPolicy<G> for NoneUCT {
    type State = ();

    fn uct_parent_factor(&self, _: &SelectionPolicyContext<G>) {}

    fn uct_child_factor(&self, _: &SelectionPolicyContext<G>, _: &Node<G>, _: &Self::State) -> f32 {
        0f32
    }
}

#[derive(Default, Copy, Clone)]
pub struct UCB1;

impl<G: Game> SelectionPolicy<G> for UCB1 {
    type State = f32;

    fn uct_parent_factor(&self, ctx: &SelectionPolicyContext<G>) -> f32 {
        let n_parent = ctx.parent.prop.n;
        ctx.config.c * (n_parent as f32).ln_1p()
    }

    fn uct_child_factor(&self, _: &SelectionPolicyContext<G>, child: &Node<G>, factor: &f32) -> f32 {
        let n_child = child.prop.n + 1;
        (factor / (n_child as f32)).sqrt()
    }
}
