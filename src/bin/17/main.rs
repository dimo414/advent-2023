use std::cell::RefCell;
use std::collections::HashMap;
use std::ops::RangeInclusive;
use std::str::FromStr;
use anyhow::*;
use advent_2023::elapsed;

use advent_2023::euclid::{Bounds, Point, point, Vector, vector};
use advent_2023::pathfinding::{Edge, Graph};

fn main() -> Result<()> {
    let map: Map = include_str!("input.txt").parse()?;
    let crucible = Crucible{ map: &map, straight_travel: 1..=3 };
    println!("Crucible heat loss: {}", crucible.path().unwrap());

    let ultra = Crucible{ map: &map, straight_travel: 4..=10 };
    println!("UltraCrucible heat loss: {}", ultra.path().unwrap());

    Ok(())
}

#[derive(Debug)]
struct Map {
    costs: HashMap<Point, i32>,
    bounds: Bounds,
    cache: RefCell<HashMap<(Vector, Point), i32>>, // (Dir, Dest) -> CostFromEdge
}

impl Map {
    fn create(costs: HashMap<Point, i32>) -> Result<Map> {
        let bounds = Bounds::from_points(costs.keys()).context("Non-empty")?;
        Ok(Map{ costs, bounds, cache: RefCell::default() })
    }

    // Returns the cost from source to dest, i.e. the sum of the blocks between these points
    // _excluding_ source. Returns None if such a path does not exist.

    // A traversal appears to fully-populate the cache so we could also pre-construct instead of
    // memoizing it.
    fn path_cost(&self, source: Point, dest: Point) -> Option<i32> {
        // We could implement Sub on Vector, but I'm not certain it's a good API in general; here it saves a little work
        fn vec_sub(pos: Point, dir: Vector) -> Point {
            point(pos.x - dir.x, pos.y - dir.y)
        }

        fn edge_cost(dir: Vector, dest: Point, costs: &HashMap<Point, i32>, cache: &mut HashMap<(Vector, Point), i32>) -> Option<i32> {
            debug_assert_eq!(dir, dir.signum());
            let cached = cache.get(&(dir, dest)).cloned();
            if cached.is_some() { return cached; } // avoiding if let so the RefCell borrow checker is happy

            let cost = *costs.get(&dest)?; // None if dest is invalid
            let prior = vec_sub(dest, dir);
            let prior_cost = edge_cost(dir, prior, costs, cache).unwrap_or(0);
            cache.insert((dir, dest), cost + prior_cost);
            Some(cost + prior_cost)
        }

        let dir = (dest - source).signum();
        debug_assert!(dir != Vector::ZERO);
        debug_assert!(dir.x == 0 || dir.y == 0);
        let dest_cost = edge_cost(dir, dest, &self.costs,&mut self.cache.borrow_mut())?;
        let source_cost = edge_cost(dir, source, &self.costs, &mut self.cache.borrow_mut())?;
        Some(dest_cost - source_cost)
    }
}

struct Crucible<'a> {
    map: &'a Map,
    straight_travel: RangeInclusive<i32>,
}

impl<'a> Crucible<'a> {
    fn path(&self) -> Option<i32> {
        let start = (self.map.bounds.min, vector(0,0));
        let target = self.map.bounds.max;
        let goal = |d: &<Crucible<'a> as Graph>::Node| d.0 == self.map.bounds.max;
        if cfg!(feature="timing") {
            self.map.cache.borrow_mut().clear();
            // A* would normally be faster (and it is if you start e.g. in the middle of the map),
            // but because we start in the top-left and end in the bottom-right Dijkstra's covers
            // essentially the same search space as A* without as much overhead.
            elapsed!("A*", self.a_star(&start, goal, |(pos, _)| (target - *pos).grid_len() as i32));
            self.map.cache.borrow_mut().clear();
        }
        let path = elapsed!("Dijkstra's", self.dijkstras(&start, goal));
        // println!("Path:");
        // path.as_ref().unwrap().iter().for_each(|e| println!("{:?}", e));
        path.map(|v| v.iter().map(|e| e.weight()).sum::<i32>())
    }
}

impl<'a> Graph for Crucible<'a> {
    type Node = (Point, Vector);

    fn neighbors(&self, source: &Self::Node) -> Vec<Edge<Self::Node>> {
        let (pos, dir) = source;
        let dest_to_edge = |(dest, dir): Self::Node| self.map.path_cost(*pos, dest)
            .map(|c| Edge::new(c, *source, (dest, dir)));

        if *dir == Vector::ZERO {
            // Crucible is not moving (i.e it must be at the start); allow it to go in all directions
            return Vector::CARDINAL.iter()
                .flat_map(|v| self.straight_travel.clone().map(|n| (pos + (*v * n), *v)))
                .filter_map(dest_to_edge)
                .collect();
        }
        debug_assert!(Vector::CARDINAL.contains(dir));

        // Having moved straight to get to this location we can only move right or left now (a
        // different path would have moved straight more/fewer steps). This way the Node doesn't
        // need to track how many steps forward we've taken since we return all valid straight paths.
        [dir.left90(), dir.right90()].iter()
            .flat_map(|v| self.straight_travel.clone().map(|n| (pos + (*v * n), *v)))
            .filter_map(dest_to_edge)
            .collect()
    }
}

impl FromStr for Map {
    type Err = Error;

    fn from_str(s: &str) -> Result<Self> {
        let mut costs = HashMap::new();
        for (y, l) in s.lines().enumerate() {
            for (x, c) in l.chars().enumerate() {
                let pos = point(x as i32, y as i32);
                let cost = c.to_digit(10).context("Invalid")? as i32;
                costs.insert(pos, cost);
            }
        }
        Map::create(costs)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn check_input() { include_str!("input.txt").parse::<Map>().unwrap(); }

    parameterized_test::create!{ part1, (input, loss), {
        let map: Map = input.parse().unwrap();
        let crucible = Crucible{ map: &map, straight_travel: 1..=3 };
        assert_eq!(crucible.path().unwrap(), loss);
    }}
    part1! {
        e1: (include_str!("example1.txt"), 102),
        e2: (include_str!("example2.txt"), 59), // not provided
        // usually we don't test the input, but the coverage seems worthwhile here and it's pretty fast
        i: (include_str!("input.txt"), 1076),
    }

    parameterized_test::create!{ part2, (input, loss), {
        let map: Map = input.parse().unwrap();
        let ultra = Crucible{ map: &map, straight_travel: 4..=10 };
        assert_eq!(ultra.path().unwrap(), loss);
    }}
    part2! {
        e1: (include_str!("example1.txt"), 94),
        e2: (include_str!("example2.txt"), 71),
        // usually we don't test the input, but the coverage seems worthwhile here and it's pretty fast
        i: (include_str!("input.txt"), 1219),
    }
}
