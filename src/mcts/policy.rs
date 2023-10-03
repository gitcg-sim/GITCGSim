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

pub trait SelectionPolicy<G: Game>: Send + Sync {
    type ParentFactor: Copy + Clone;
    fn uct_parent_factor(&self, config: &MCTSConfig, parent: &Node<G>) -> Self::ParentFactor;
    fn uct_child_factor(
        &self,
        config: &MCTSConfig,
        parent: &Node<G>,
        child: &Node<G>,
        parent_factor: Self::ParentFactor,
    ) -> f32;
}

#[derive(Default, Copy, Clone)]
pub struct NoneUCT;
impl<G: Game> SelectionPolicy<G> for NoneUCT {
    type ParentFactor = ();

    fn uct_parent_factor(&self, _: &MCTSConfig, _: &Node<G>) {}

    fn uct_child_factor(&self, _: &MCTSConfig, _: &Node<G>, _: &Node<G>, _: ()) -> f32 {
        0f32
    }
}

#[derive(Default, Copy, Clone)]
pub struct UCB1;

impl<G: Game> SelectionPolicy<G> for UCB1 {
    type ParentFactor = f32;

    fn uct_parent_factor(&self, config: &MCTSConfig, parent: &Node<G>) -> f32 {
        let n_parent = parent.prop.n;
        config.c * (n_parent as f32).ln_1p()
    }

    fn uct_child_factor(&self, _: &MCTSConfig, _: &Node<G>, child: &Node<G>, parent_factor: f32) -> f32 {
        let n_child = child.prop.n + 1;
        (parent_factor / (n_child as f32)).sqrt()
    }
}
