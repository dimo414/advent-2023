use std::collections::HashMap;
use std::rc::Rc;
use std::str::FromStr;
use anyhow::*;
use itertools::Itertools;

use advent_2023::pathfinding::{Edge, Graph, NodeGraph};

// Didn't implement https://en.wikipedia.org/wiki/Karger%27s_algorithm (per se), but it's related
fn main() -> Result<()> {
    let args: Vec<_> = std::env::args().skip(1).collect();
    if !args.is_empty() {
        let passes: usize = args[0].parse()?;
        let max: usize = args.get(1).map(|p| p.parse()).unwrap_or(Result::Ok(100))?;
        let step_by: usize = args.get(2).map(|p| p.parse()).unwrap_or(Result::Ok(5))?;
        return trials(passes, max, step_by);
    }

    let mut input: Components = include_str!("input.txt").parse()?;

    if cfg!(debug_assertions) {
        let path = "25.gv";
        std::fs::write(path, input.graphviz_undirected()?).expect("Unable to write DOT file");
        println!("Wrote DOT file to {}", path);
    }

    // From the above tests, 100 passes seems to yield the correct answer 99.9+% of the time
    let edges = input.candidate_edges(100);
    assert_eq!(edges.len(), 3);
    input.remove_edges(&edges);

    let forest = input.forest();
    assert_eq!(forest.len(), 2, "Expected two disjoint groups of nodes");

    println!("Product of component sizes: {}", forest.iter().map(|v| v.len()).product::<usize>());

    Ok(())
}

fn trials(passes: usize, max: usize, step_by: usize) -> Result<()> {
    for n in (step_by..=max).step_by(step_by) {
        let mut valid = 0;
        for _ in 0..passes {
            let mut input: Components = include_str!("input.txt").parse()?;
            let edges = input.candidate_edges(n);
            input.remove_edges(&edges);
            let forest = input.forest();
            if forest.len() == 2 {
                valid += 1;
            }
        }
        println!("{} traversals succeeded {}/{} times: {:.1}%", n, valid, passes, 100.0 * valid as f64 / passes as f64);
    }
    Ok(())
}

#[derive(Debug)]
struct Components {
    edges: HashMap<Rc<str>, Vec<Rc<str>>>,
}

impl Components {
    fn create(connections: HashMap<&str, Vec<&str>>) -> Components {
        let mut components = Components{ edges: HashMap::new() };
        for (source, dests) in connections {
            let source = components.intern(source);
            for dest in dests {
                let dest = components.intern(dest);
                components.edges.get_mut(&source).expect("Interned").push(dest.clone());
                components.edges.get_mut(&dest).expect("Interned").push(source.clone());
            }
        }
        components
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

    fn necessary_edges(&self, start: &str) -> HashMap<(Rc<str>, Rc<str>), usize> {
        let paths = self.bfs_all(&Rc::from(start));
        let mut counts = HashMap::new();
        for edge in paths.values().flat_map(|route| route.windows(2)) {
            let (source, dest) = if edge[0] < edge[1] { (&edge[0], &edge[1]) } else { (&edge[1], &edge[0]) };
            // TODO can we avoid clone()-ing for entries that already exist?
            *counts.entry((source.clone(), dest.clone())).or_insert(0) += 1;
        }
        counts
    }

    fn repeated_necessary_edges(&self, n: usize) -> HashMap<(Rc<str>, Rc<str>), usize> {
        let mut ret = HashMap::new();
        // This relies on HashMap iteration order being sufficiently random; this should be a safe
        // assumption given existing implementation (notably
        for (edge, count) in self.edges.keys().take(n).flat_map(|node| self.necessary_edges(node.as_ref()).into_iter()) {
            *ret.entry(edge).or_insert(0) += count;
        }
        ret
    }

    fn candidate_edges(&self, n: usize) -> Vec<(Rc<str>, Rc<str>)> {
        self.repeated_necessary_edges(n).into_iter()
            .sorted_by_key(|(_, v)| *v).rev()
            .take(3)
            .map(|(k, _)| k)
            .collect()
    }

    fn remove_edge(&mut self, source: &str, dest: &str) {
        let mut rm_dir = |source: &str, dest: &str| {
            let dirs = self.edges.get_mut(source).expect("No edge");
            let idx = dirs.iter().position(|v| v.as_ref() == dest).expect("No idx");
            assert_eq!(dirs.swap_remove(idx).as_ref(), dest);
        };
        rm_dir(source, dest);
        rm_dir(dest, source);
    }

    fn remove_edges(&mut self, edges: &[(Rc<str>, Rc<str>)]) {
        for (source, dest) in edges {
            self.remove_edge(source.as_ref(), dest.as_ref());
        }
    }
}

impl Graph for Components {
    type Node = Rc<str>;

    fn neighbors(&self, source: &Self::Node) -> Vec<Edge<Self::Node>> {
        self.edges.get(source).into_iter()
            .flat_map(|dests| dests.iter()
                .map(|d| Edge::new(1, source.clone(), d.clone())))
            .collect()
    }
}

impl NodeGraph for Components {
    fn nodes(&self) -> Vec<Self::Node> {
        self.edges.keys().cloned().collect()
    }
}

impl FromStr for Components {
    type Err = Error;

    fn from_str(s: &str) -> Result<Self> {
        let connections = s.lines().map(|l| {
            let (source, dests) = l.split(": ").collect_tuple().context("Invalid")?;
            let dests: Vec<_> = dests.split(' ').collect();
            Ok((source, dests))
        })
            .collect::<Result<HashMap<_,_>>>()?;

        Ok(Components::create(connections))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn check_input() { include_str!("input.txt").parse::<Components>().unwrap(); }

    #[test]
    fn example() {
        let mut input: Components = include_str!("example.txt").parse().unwrap();

        // 100 passes seems mostly sufficient to avoid false positives
        let edges = input.candidate_edges(100);
        assert_eq!(edges.len(), 3);
        for (source, dest) in edges {
            input.remove_edge(source.as_ref(), dest.as_ref());
        }

        let forest = input.forest();
        let sizes: Vec<_> = forest.iter().map(|n| n.len()).sorted().collect();
        assert_eq!(sizes, [6, 9]);
    }
}
