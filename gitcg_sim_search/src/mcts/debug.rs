use super::*;
use crate::*;

#[derive(Debug, Default)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct TreeDump<T> {
    #[cfg_attr(feature = "serde", serde(rename = "_"))]
    pub value: T,
    #[cfg_attr(feature = "serde", serde(rename = "children"))]
    pub children: Vec<Rc<TreeDump<T>>>,
}

impl<T> TreeDump<T> {
    pub fn new(value: T, children: Vec<Rc<TreeDump<T>>>) -> Self {
        Self { value, children }
    }
}

impl Proportion {
    fn format_ratio(self: Proportion) -> String {
        let p = self;
        let r = p.ratio();
        format!("{p} = {:.2}% \u{b1} {:.2}%", 1e2 * r, 1e2 * 2.0 * p.sd())
    }
}

impl<G: Game> NodeData<G> {
    fn debug_description(&self, children_count: usize, describe_action: &dyn Fn(G::Action) -> String) -> String {
        let action = self.action;
        let action_part = if let Some(action) = action {
            describe_action(action)
        } else {
            "[Root]".to_string()
        };

        let stats_part: NodeStats = self.last_stats.lock().map(|s| s.clone()).unwrap_or_default();
        format!(
            "{action_part} ({}), #children = {}, depth={}, stats={stats_part:?}",
            self.prop.format_ratio(),
            children_count,
            self.depth
        )
    }
}

impl<G: Game, E: EvalPolicy<G>, S: SelectionPolicy<G>> MCTS<G, E, S> {
    pub fn dump_tree(
        &self,
        token: Token,
        max_depth: u8,
        describe_action: &dyn Fn(G::Action) -> String,
    ) -> TreeDump<String> {
        if max_depth == 0 {
            return Default::default();
        }

        let Some(node) = self.tree.get(token) else {
            return Default::default();
        };

        let children_count = node.children(&self.tree).count();
        let desc = node.data.debug_description(children_count, describe_action);
        let max_depth_1 = max_depth - 1;
        let children = node
            .children(&self.tree)
            .map(|child| Rc::new(self.dump_tree(child.token(), max_depth_1, describe_action)))
            .filter(|child| !(child.children.is_empty() && child.value.is_empty()))
            .collect();
        TreeDump::new(desc, children)
    }

    pub fn print_tree(&self, token: Token, depth: u8, max_depth: u8, min_n: u32) {
        if depth > max_depth {
            return;
        }

        let Some(node) = self.tree.get(token) else {
            return;
        };

        fn indent_prefix(indent_depth: u8) -> String {
            let mut s = Default::default();
            if indent_depth == 0 {
                s += "- ";
                return s;
            }
            for _ in 0..indent_depth {
                s += "  ";
            }
            s += "- ";
            s
        }

        let node_part: String = node
            .data
            .debug_description(node.children(&self.tree).count(), &|a| format!("{a:?}"));
        println!("{}{}", indent_prefix(depth), node_part);
        let mut omitted_prop = Proportion::default();
        let mut omitted = 0;
        let mut found = false;
        let mut children: SmallVec<[_; 16]> = node.children(&self.tree).collect();
        let is_maximize = node.data.is_maximize(self.maximize_player);
        children.sort_by_cached_key(|c| (-1e6 * c.data.ratio(is_maximize)) as i32);
        let c = children.len();
        for (i, child) in children.iter().copied().enumerate() {
            let NodeData { prop, .. } = child.data;
            let n = prop.n;
            if n != 0 && (c <= 1 || depth == 0 || n >= min_n || i == 0) {
                found = depth < max_depth;
                self.print_tree(child.token(), depth + 1, max_depth, min_n);
            } else {
                omitted += 1;
                omitted_prop += prop;
            }
        }

        if found && omitted > 0 {
            println!(
                "{}...[{omitted} omitted] ({})",
                indent_prefix(depth + 1),
                omitted_prop.format_ratio()
            );
        }
    }
}
