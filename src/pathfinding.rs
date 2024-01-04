mod internal {
    use std::collections::{VecDeque, BinaryHeap, HashSet};
    use std::cmp::Ordering;
    use std::fmt::Debug;
    use std::fmt::Write;
    use std::hash::Hash;
    use ahash::{AHashMap, AHashSet};
    use anyhow::{ensure, Result};
    use crate::collect::DisjointSet;

    // References:
    // https://www.redblobgames.com/pathfinding/a-star/introduction.html
    // http://theory.stanford.edu/~amitp/GameProgramming/AStarComparison.html
    // https://doc.rust-lang.org/std/collections/binary_heap/
    pub trait Graph {
        // TODO at least sometimes the default Hash impl is surprisingly expensive (see Day 23).
        //    Should benchmark using BTreeMap/Set to see if that's preferable, or look into revising
        //    the Hash impls of all Nodes.
        type Node: Clone + Debug + Eq + Hash;

        fn neighbors(&self, source: &Self::Node) -> Vec<Edge<Self::Node>>;

        fn bfs_all(&self, start: &Self::Node) -> AHashMap<Self::Node, Vec<Self::Node>> {
            let mut frontier = VecDeque::new();
            frontier.push_back(start.clone());
            let mut routes = AHashMap::new();
            routes.insert(start.clone(), start.clone()); // careful, potential infinite loop

            while ! frontier.is_empty() {
                let current = frontier.pop_front().expect("frontier is not empty");
                for edge in self.neighbors(&current) {
                    assert!(edge.weight() == 1, "BFS does not support weighted edges: {:?}", edge);
                    let next = edge.dest();
                    if !routes.contains_key(next) {
                        frontier.push_back(next.clone());
                        routes.insert(next.clone(), current.clone());
                    }
                }
            }

            let mut paths = AHashMap::new();
            'outer: for goal in routes.keys() {
                let mut path = Vec::new();
                let mut current = goal.clone();
                while current != *start {
                    if let Some(next) = routes.get(&current) {
                        path.push(current);
                        current = next.clone();
                    } else {
                        continue 'outer;
                    }
                }
                path.push(start.clone());
                path.reverse();
                paths.insert(goal.clone(), path);
            }
            paths
        }

        fn bfs(&self, start: &Self::Node, mut goal_predicate: impl FnMut(&Self::Node) -> bool) -> Option<Vec<Self::Node>> {
            let mut frontier = VecDeque::new();
            frontier.push_back(start.clone());
            let mut routes = AHashMap::new();
            let mut goal = None;

            while let Some(current) = frontier.pop_front() {
                if goal_predicate(&current) {
                    goal = Some(current.clone());
                    break;
                }
                for edge in self.neighbors(&current) {
                    assert!(edge.weight() == 1, "BFS does not support weighted edges: {:?}", edge);
                    let next = edge.dest();
                    if !routes.contains_key(next) {
                        frontier.push_back(next.clone());
                        routes.insert(next.clone(), current.clone());
                    }
                }
            }

            let mut current = goal?;
            let mut path = Vec::new();
            while current != *start {
                if let Some(next) = routes.get(&current) {
                    path.push(current);
                    current = next.clone();
                } else {
                    unreachable!();
                }
            }
            path.push(start.clone());
            path.reverse();
            Some(path)
        }

        fn dijkstras(&self, start: &Self::Node, mut goal_predicate: impl FnMut(&Self::Node) -> bool) -> Option<Vec<Edge<Self::Node>>> {
            let mut frontier = BinaryHeap::new();
            let mut visited = AHashSet::new();
            let mut costs = AHashMap::new();
            let mut routes = AHashMap::new();
            let mut goal = None;
            frontier.push(State { cost: 0, node: start.clone() });
            costs.insert(start.clone(), 0);

            while let Some(current) = frontier.pop() {
                if goal_predicate(&current.node) {
                    goal = Some(current.node.clone());
                    break;
                }
                if visited.contains(&current.node) { continue; }
                visited.insert(current.node.clone());
                debug_assert_eq!(Some(&current.cost), costs.get(&current.node));
                for edge in self.neighbors(&current.node) {
                    let next = edge.dest();
                    let next_cost = current.cost + edge.weight();

                    let prior_next_cost = costs.get(next);
                    if prior_next_cost.is_none() || *prior_next_cost.expect("Not-none") > next_cost {
                        costs.insert(next.clone(), next_cost);
                        frontier.push(State { cost: next_cost, node: next.clone() });
                        routes.insert(next.clone(), edge.clone());
                    }
                }
            }

            let mut current = goal?;
            let mut path = Vec::new();
            while current != *start {
                if let Some(next) = routes.get(&current) {
                    path.push(next.clone());
                    current = next.source().clone();
                } else {
                    unreachable!();
                }
            }
            path.reverse();
            Some(path)
        }

        fn dijkstras_all(&self, start: &Self::Node) -> AHashMap<Self::Node, Vec<Edge<Self::Node>>> {
            let mut frontier = BinaryHeap::new();
            let mut costs = AHashMap::new();
            let mut routes = AHashMap::new();
            frontier.push(State { cost: 0, node: start.clone() });
            costs.insert(start.clone(), 0);
            routes.insert(start.clone(),
                          Edge::new(0, start.clone(), start.clone())); // careful, potential infinite loop

            while let Some(current) = frontier.pop() {
                debug_assert_eq!(Some(&current.cost), costs.get(&current.node));
                for edge in self.neighbors(&current.node) {
                    let next = edge.dest();
                    let next_cost = current.cost + edge.weight();

                    let prior_next_cost = costs.get(next);
                    if prior_next_cost.is_none() || *prior_next_cost.expect("Not-none") > next_cost {
                        costs.insert(next.clone(), next_cost);
                        frontier.push(State { cost: next_cost, node: next.clone() });
                        routes.insert(next.clone(), edge.clone());
                    }
                }
            }

            let mut paths = AHashMap::new();
            for goal in routes.keys() {
                let mut path = Vec::new();
                let mut current = goal.clone();
                while current != *start {
                    if let Some(next) = routes.get(&current) {
                        path.push(next.clone());
                        current = next.source().clone();
                    } else {
                        unreachable!();
                    }
                }
                path.reverse();
                paths.insert(goal.clone(), path);
            }
            paths
        }

        fn a_star(&self, start: &Self::Node, mut goal_predicate: impl FnMut(&Self::Node) -> bool, heuristic: impl Fn(&Self::Node) -> i32) -> Option<Vec<Edge<Self::Node>>> {
            let mut frontier = BinaryHeap::new();
            let mut visited = AHashSet::new();
            let mut costs = AHashMap::new();
            let mut est_costs = AHashMap::new();
            let mut routes = AHashMap::new();
            let mut goal = None;
            let start_state = EstState { est_cost: heuristic(start), real_cost: 0, node: start.clone() };
            costs.insert(start.clone(), start_state.real_cost);
            est_costs.insert(start.clone(), start_state.est_cost);
            frontier.push(start_state);

            while let Some(current) = frontier.pop() {
                if goal_predicate(&current.node) {
                    goal = Some(current.node.clone());
                    break;
                }

                if visited.contains(&current.node) { continue; }
                visited.insert(current.node.clone());
                debug_assert_eq!(Some(&current.est_cost), est_costs.get(&current.node));
                debug_assert_eq!(Some(&current.real_cost), costs.get(&current.node));
                for edge in self.neighbors(&current.node) {
                    let next = edge.dest();
                    let next_cost = current.real_cost + edge.weight();

                    let prior_next_cost = costs.get(next);
                    if prior_next_cost.is_none() || *prior_next_cost.expect("Not-none") > next_cost {
                        let next_state = EstState { est_cost: next_cost + heuristic(next), real_cost: next_cost, node: next.clone() };
                        costs.insert(next.clone(), next_state.real_cost);
                        est_costs.insert(next.clone(), next_state.est_cost);
                        frontier.push(next_state);
                        routes.insert(next.clone(), edge);
                    }
                }
            }

            let mut current = goal?;
            let mut path = Vec::new();
            while current != *start {
                if let Some(next) = routes.get(&current) {
                    path.push(next.clone());
                    current = next.source().clone();
                } else {
                    unreachable!();
                }
            }
            path.reverse();
            Some(path)
        }
    }

    #[derive(Copy, Clone, Debug)]
    pub struct Edge<N: Clone + Debug> {
        weight: i32,
        source: N,
        dest: N,
    }

    impl<N: Clone + Debug> Edge<N> {
        pub fn new(weight: i32, source: N, dest: N) -> Edge<N> {
            Edge { weight, source, dest }
        }

        pub fn weight(&self) -> i32 { self.weight }
        pub fn source(&self) -> &N { &self.source }
        pub fn dest(&self) -> &N { &self.dest }
    }

    #[derive(Copy, Clone, Debug)]
    struct State<N: Clone + Debug> {
        cost: i32,
        node: N,
    }

    // We don't implement Eq because it's not well defined, but Ord requires it exist
    impl<N: Clone + Debug> PartialEq for State<N> {
        fn eq(&self, _: &Self) -> bool {
            unimplemented!()
        }
    }

    impl<N: Clone + Debug> Eq for State<N> {}

    impl<N: Clone + Debug> Ord for State<N> {
        fn cmp(&self, other: &State<N>) -> Ordering {
            other.cost.cmp(&self.cost)
        }
    }

    impl<N: Clone + Debug> PartialOrd for State<N> {
        fn partial_cmp(&self, other: &State<N>) -> Option<Ordering> {
            Some(self.cmp(other))
        }
    }

    #[derive(Copy, Clone, Debug)]
    struct EstState<N: Clone + Debug> {
        est_cost: i32,
        real_cost: i32,
        node: N,
    }

    // We don't implement Eq because it's not well defined, but Ord requires it exist
    impl<N: Clone + Debug> PartialEq for EstState<N> {
        fn eq(&self, _: &Self) -> bool {
            unimplemented!()
        }
    }

    impl<N: Clone + Debug> Eq for EstState<N> {}

    impl<N: Clone + Debug> Ord for EstState<N> {
        fn cmp(&self, other: &EstState<N>) -> Ordering {
            other.est_cost.cmp(&self.est_cost)
        }
    }

    impl<N: Clone + Debug> PartialOrd for EstState<N> {
        fn partial_cmp(&self, other: &EstState<N>) -> Option<Ordering> {
            Some(self.cmp(other))
        }
    }

    pub trait NodeGraph: Graph {
        fn nodes(&self) -> Vec<Self::Node>;

        // http://magjac.com/graphviz-visual-editor/
        // https://dreampuf.github.io/GraphvizOnline/
        fn graphviz_directed(&self) -> String {
            let mut out = String::new();
            out.push_str("digraph G {\n");
            for node in self.nodes() {
                writeln!(out, "{:?}", node).expect("Impossible");
                for edge in self.neighbors(&node) {
                    if edge.weight != 1 {
                        writeln!(out, "{:?} -> {:?} [label={}]", edge.source(), edge.dest(), edge.weight()).expect("Impossible");
                    } else {
                        writeln!(out, "{:?} -> {:?}", edge.source(), edge.dest()).expect("Impossible");
                    }
                }
            }

            out.push_str("}\n");
            out
        }

        fn graphviz_undirected(&self) -> Result<String> {
            let mut out = String::new();
            let mut seen = HashSet::new();
            out.push_str("graph G {\n");
            for node in self.nodes() {
                writeln!(out, "{:?}", node).expect("Impossible");
                for edge in self.neighbors(&node) {
                    if !seen.remove(&(edge.dest().clone(), edge.source().clone())) {
                        if edge.weight != 1 {
                            writeln!(out, "{:?} -- {:?} [label={}]", edge.source(), edge.dest(), edge.weight()).expect("Impossible");
                        } else {
                            writeln!(out, "{:?} -- {:?}", edge.source(), edge.dest()).expect("Impossible");
                        }
                        seen.insert((edge.source().clone(), edge.dest().clone()));
                    }
                }
            }
            ensure!(seen.is_empty(), "Unbalanced edges, graph is not undirected: {:?}", seen);

            out.push_str("}\n");
            Ok(out)
        }

        fn spanning_tree(&self) -> Result<Vec<Edge<Self::Node>>> {
            let mut ret = Vec::new();
            let mut edges = self.nodes().iter().flat_map(|n| self.neighbors(n)).collect::<Vec<_>>();
            edges.sort_by_key(|e| e.weight());
            let edges = edges;

            let nodes = self.nodes();
            let num_nodes = nodes.len();
            let mut sets = DisjointSet::create(nodes);
            for edge in edges {
                let source = edge.source();
                let dest = edge.dest();
                if sets.union(source, dest) { // critical edge
                    let cur_set_size = sets.set_size(source);
                    ret.push(edge);
                    if cur_set_size == num_nodes { break; }
                }
            }
            let roots = sets.roots();
            ensure!(roots.len() == 1, "Disjoint graph: {:?}", roots);
            Ok(ret)
        }

        // TBD if this is needed vs. what DisjointSet can do
        fn forest(&self) -> Vec<HashSet<Self::Node>> {
            let mut ret = Vec::new();
            let mut unseen: HashSet<_> = self.nodes().into_iter().collect();

            while !unseen.is_empty() {
                let source = unseen.iter().next().expect("Non-empty");
                let mut frontier = VecDeque::from([source.clone()]);
                let mut cur_tree = HashSet::new();
                while let Some(current) = frontier.pop_front() {
                    if cur_tree.contains(&current) { continue; }
                    assert!(unseen.remove(&current), "{:?} is already in another tree ({:?}); is this graph undirected?", current, ret);
                    for edge in self.neighbors(&current) {
                        frontier.push_back(edge.dest);
                    }
                    cur_tree.insert(current);
                }
                ret.push(cur_tree);
            }

            ret
        }
    }
}
pub use self::internal::{Edge,Graph,NodeGraph};

#[cfg(test)]
mod tests {
    use super::*;
    use crate::euclid::{point, Point, Vector};
    use std::collections::{BTreeMap, HashSet};
    use std::rc::Rc;
    use ahash::{AHashMap, AHashSet};
    use itertools::Itertools;

    struct BasicGraph {
        blocked: AHashSet<Point>,
    }

    impl BasicGraph {
        fn new(blocked: &[Point]) -> BasicGraph {
            BasicGraph { blocked: blocked.iter().cloned().collect() }
        }
    }

    impl Graph for BasicGraph {
        type Node = Point;

        fn neighbors(&self, source: &Self::Node) -> Vec<Edge<Self::Node>> {
            if self.blocked.contains(source) { return vec!(); }

            Vector::CARDINAL.iter()
                .map(|v| source + v)
                .filter(|p| !self.blocked.contains(p))
                .map(|d| Edge::new(1, *source, d))
                .collect()
        }
    }

    #[test]
    fn direct() {
        let graph = BasicGraph::new(&[]);
        let start = point(1, 1);
        let goal = point(3, 4);

        let bfs_route = graph.bfs(&start, |n| n == &goal).unwrap();
        assert_eq!(bfs_route.len(), 6);
        assert_eq!(bfs_route[0], start);
        assert_eq!(bfs_route[bfs_route.len()-1], goal);

        let djk_route = graph.dijkstras(&start, |n| n == &goal).unwrap();
        assert_eq!(djk_route.len(), 5);
        assert_eq!(djk_route[0].source(), &start);
        assert_eq!(djk_route[djk_route.len()-1].dest(), &goal);

        let as_route = graph.a_star(&start, |n| n == &goal, |n| (goal - *n).grid_len() as i32).unwrap();
        assert_eq!(as_route.len(), 5);
        assert_eq!(as_route[0].source(), &start);
        assert_eq!(as_route[djk_route.len()-1].dest(), &goal);
    }

    #[test]
    fn wall() {
        let graph = BasicGraph::new(&[
            point(0, 3), point(1, 3), point(2, 3), point(3, 3), point(4, 3)
        ]);
        let start = point(1, 1);
        let goal = point(3, 4);

        let bfs_route = graph.bfs(&start, |n| n == &goal).unwrap();
        assert_eq!(bfs_route.len(), 10);
        assert_eq!(bfs_route[0], start);
        assert_eq!(bfs_route[bfs_route.len()-1], goal);

        let djk_route = graph.dijkstras(&start, |n| n == &goal).unwrap();
        assert_eq!(djk_route.len(), 9);
        assert_eq!(djk_route[0].source(), &start);
        assert_eq!(djk_route[djk_route.len()-1].dest(), &goal);

        let as_route = graph.a_star(&start, |n| n == &goal, |n| (goal - *n).grid_len() as i32).unwrap();
        assert_eq!(as_route.len(), 9);
        assert_eq!(as_route[0].source(), &start);
        assert_eq!(as_route[djk_route.len()-1].dest(), &goal);
    }

    #[test]
    fn all_paths() {
        // From 2019 Day 15 pt 2 - forms a small room
        let graph = BasicGraph::new(&[
            point(1,0), point(2, 0),
            point(0, 1), point(3, 1), point(4, 1),
            point(0, 2), point(2, 2), point(5, 2),
            point(0, 3), point(4, 3),
            point(1, 4), point(2, 4), point(3, 4)
        ]);
        let start = point(2, 3);
        let farthest = point(2, 1);

        let bfs_routes = graph.bfs_all(&start);
        let djk_routes = graph.dijkstras_all(&start);

        let bfs_routes_lens: BTreeMap<_,_> = bfs_routes.iter().map(|(&k, v)| (k, v.len() as i32 - 1)).collect();
        let djk_routes_lens: BTreeMap<_,_> = djk_routes.iter()
            .map(|(&k, v)| (k, v.iter().map(|e| e.weight()).sum())).collect();
        let expected_routes: BTreeMap<_,_> = [
            (point(1, 1), 3), (point(2, 1), 4), (point(1, 2), 2), (point(3, 2), 2),
            (point(4, 2), 3), (point(1, 3), 1), (point(2, 3), 0), (point(3, 3), 1)
        ].into_iter().collect();
        assert_eq!(bfs_routes_lens, expected_routes);
        assert_eq!(djk_routes_lens, expected_routes);

        let bfs_route = graph.bfs(&start, |n| n == &farthest).unwrap();
        let bfs_all_route = bfs_routes.get(&farthest).unwrap();
        assert_eq!(bfs_route.len(), bfs_all_route.len());
        // This is not strictly true, but there's only one route to this point for this graph,
        // so it should be reliable for this test case
        assert_eq!(&bfs_route, bfs_all_route);
    }

    struct RcGraph {
        edges: AHashMap<Rc<str>, Vec<(Rc<str>, i32)>>,
    }

    impl RcGraph {
        fn create<'a>(all_edges: impl IntoIterator<Item=(&'a str, &'a str, i32)>) -> RcGraph {
            let mut graph = RcGraph{ edges: AHashMap::new() };
            for (source, dest, weight) in all_edges {
                let source = graph.intern(source);
                let dest = graph.intern(dest);
                graph.edges.get_mut(&source).expect("Interned").push((dest, weight));
            }
            graph
        }

        fn undirected<'a>(all_edges: impl IntoIterator<Item=(&'a str, &'a str)>) -> RcGraph {
            RcGraph::create(all_edges.into_iter().flat_map(|(s, d)| [(s, d, 1), (d, s, 1)].into_iter()))
        }

        fn directed<'a>(all_edges: impl IntoIterator<Item=(&'a str, &'a str)>) -> RcGraph {
            RcGraph::create(all_edges.into_iter().map(|(s, d)| (s, d, 1)))
        }

        fn intern(&mut self, node: &str) -> Rc<str> {
            match self.edges.get_key_value(node) {
                Some((k, _)) => k.clone(),
                None => {
                    let node: Rc<str> = Rc::from(node);
                    self.edges.insert(node.clone(), Vec::new());
                    node
                }
            }
        }
    }

    impl Graph for RcGraph {
        type Node = Rc<str>;

        fn neighbors(&self, source: &Self::Node) -> Vec<Edge<Self::Node>> {
            self.edges.get(source).into_iter()
                .flat_map(|dest| dest.iter().map(|(d,w)| Edge::new(*w, source.clone(), d.clone())))
                .collect()
        }
    }

    impl NodeGraph for RcGraph {
        fn nodes(&self) -> Vec<Self::Node> {
            self.edges.keys().cloned().collect()
        }
    }

    #[test]
    fn refcounted() {
        let mut graph = RcGraph::directed([("A", "B"), ("B", "C"), ("C", "D")]);
        let start = graph.intern("A");
        let bfs_route = graph.bfs(&start, |n| n.as_ref() == "D").unwrap();

        assert_eq!(bfs_route.len(), 4);
        assert_eq!(bfs_route.first().unwrap().as_ref(), "A");
        assert_eq!(bfs_route.last().unwrap().as_ref(), "D");
    }
    
    #[test]
    fn forest() {
        let graph = RcGraph::undirected([("A", "B"), ("B", "C"), ("D", "E")]);
        // sort the vec by length to ensure consistent ordering for the assertions
        let forest = graph.forest().into_iter().sorted_by_key(|v| v.len()).collect_vec();
        assert_eq!(forest, &[
            HashSet::from([Rc::from("D"), Rc::from("E")]),
            HashSet::from([Rc::from("A"), Rc::from("B"), Rc::from("C")])]);
    }

    #[test]
    fn spanning_tree() {
        let graph = RcGraph::create([("A", "B", 1), ("B", "A", 2), ("B", "C", 3), ("A", "B", 4), ("D", "E", 5), ("C", "D", 6)]);
        let edges = graph.spanning_tree().unwrap()
            .into_iter().map(|e| (e.source().to_string(), e.dest().to_string(), e.weight())).collect::<Vec<_>>();
        println!("{:?}", edges);
        assert_eq!(edges, [
            ("A".to_string(), "B".to_string(), 1),
            ("B".to_string(), "C".to_string(), 3),
            ("D".to_string(), "E".to_string(), 5),
            ("C".to_string(), "D".to_string(), 6)]);
    }

    #[test]
    fn graphviz() {
        // Not bothering to validate the syntax for now, just check the calls work
        let graph = RcGraph::undirected([("A", "B"), ("B", "C"), ("D", "E")]);
        let directed = graph.graphviz_directed();
        assert!(directed.contains(" -> "));
        let undirected = graph.graphviz_undirected().unwrap();
        assert!(undirected.contains(" -- "));
    }
}
