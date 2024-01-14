pub mod linked_list;
pub use linked_list::*;

pub mod game_trait;
pub use game_trait::*;

pub mod game_state_wrapper;
pub use game_state_wrapper::*;

pub mod transposition_table;

/// Implementation for Monte-Carlo Tree Search
pub mod mcts;

pub mod rule_based {
    use gitcg_sim::{prelude::*, rule_based::*};

    use super::*;

    impl<S: NondetState> GameTreeSearch<GameStateWrapper<S>> for RuleBasedSearch {
        fn search(
            &mut self,
            position: &GameStateWrapper<S>,
            maximize_player: PlayerId,
        ) -> SearchResult<GameStateWrapper<S>> {
            let (action_scores, i) = self.search_and_select(position, maximize_player);
            let mut counter = SearchCounter::default();
            counter.states_visited += 1;
            SearchResult {
                pv: linked_list![action_scores[i].0],
                eval: Default::default(),
                counter,
            }
        }
    }
}

/// Implementation for minimax search
pub mod minimax;

pub mod playout;

pub mod training;

pub mod search;
pub use search::*;

pub mod prelude {
    pub use crate::linked_list::*;
    pub use crate::{EvalTrait, Game, GameTreeSearch, SearchCounter, SearchLimits, SearchResult};
}
