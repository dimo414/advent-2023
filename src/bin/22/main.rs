use std::collections::{BTreeMap, HashMap, HashSet, VecDeque};
use std::str::FromStr;
use anyhow::*;
use itertools::Itertools;
use lazy_regex::regex_captures;

use advent_2023::collect::MoreIntoIterator;
use advent_2023::elapsed;
use advent_2023::euclid::{Bounds as Bounds2d, point as point2d};

fn main() -> Result<()> {
    let mut input = parse_input(include_str!("input.txt"))?;
    elapsed!(input.descend_all());

    if std::env::args().len() > 1 {
        println!("{}", input.openscad());
        return Ok(())
    }

    let critical = input.critical_bricks();
    println!("Non-critical bricks: {}", input.non_critical_bricks(&critical).count());

    if cfg!(feature="timing") {
        println!("Unstable bricks (simulated): {}", elapsed!(input.simulate_unstable_bricks()).values().sum::<usize>());
    }

    println!("Unstable bricks: {}", elapsed!(input.find_all_unstable()).values().map(|v| v.len()).sum::<usize>());

    Ok(())
}

struct Tower {
    bricks: BTreeMap<i32, HashSet<Brick>>,
    supported_by: HashMap<usize, Vec<usize>>,
    supports: HashMap<usize, Vec<usize>>,
}

impl Tower {
    fn create(all_bricks: impl IntoIterator<Item=BrickStr>) -> Tower {
        let mut bricks = BTreeMap::new();
        for (i, brick) in all_bricks.into_iter().enumerate() {
            let brick = Brick::create(i+1, brick);
            bricks.entry(brick.z_top).or_insert_with(|| HashSet::new()).insert(brick);
        }
        Tower{ bricks, supported_by: HashMap::new(), supports: HashMap::new() }
    }

    fn clone_without(&self, id: usize) -> Tower {
        let mut bricks = BTreeMap::new();
        let all_bricks = self.bricks.values().flat_map(|s| s.iter())
            .filter(|b| b.id != id);
        for brick in all_bricks {
            bricks.entry(brick.z_top).or_insert_with(|| HashSet::new()).insert(brick.clone());
        }
        Tower{ bricks, supported_by: HashMap::new(), supports: HashMap::new() }
    }

    fn descend_all(&mut self) -> usize {
        let mut moved = 0;
        let top_row = self.bricks.last_key_value().map(|(k,_)| *k).unwrap();
        for i in 2..=top_row {
            moved += self.descend_row(i);
        }

        for (k, v) in &self.supported_by {
            for v in v {
                self.supports.entry(*v).or_insert_with(|| Vec::new()).push(*k);
            }
        }

        moved
    }

    // Important: this assumes all bricks below row are in their final positions
    fn descend_row(&mut self, row: i32) -> usize {
        let mut moved = 0;
        let bricks = self.bricks.remove(&row).unwrap_or_else(|| HashSet::new());
        for mut brick in bricks {
            debug_assert_eq!(brick.z_top, row);
            self.descend_brick(&mut brick);
            if brick.z_top != row { moved += 1; }
            self.bricks.entry(brick.z_top).or_insert_with(|| HashSet::new()).insert(brick);
        }
        moved
    }

    fn descend_brick(&mut self, brick: &mut Brick) {
        // can't descend lower than height, which is 1 for horizontal bricks
        for row in (brick.height..brick.z_top).rev() {
            // look for collisions in the bottom row of the brick's height
            let collisions = self.collisions(brick.bounds2d, row-brick.height+1);
            if !collisions.is_empty() {
                brick.z_top = row+1; // stay in the row above if there are collisions here
                let prior = self.supported_by.insert(brick.id, collisions);
                debug_assert!(prior.is_none());
                return;
            }
        }
        brick.z_top = brick.height; // nothing collided so stop at the floor
    }

    fn collisions(&self, bounds2d: Bounds2d, row: i32) -> Vec<usize> {
        self.bricks.get(&row).iter().flat_map(|s| s.iter())
            .filter(|b| b.bounds2d.intersects(bounds2d))
            .map(|b| b.id)
            .collect()
    }

    // Bricks that uniquely support at least one brick
    fn critical_bricks(&self) -> HashSet<usize> {
        self.supported_by.values()
            .filter(|s| s.len() == 1)
            .map(|v| *v.take_only().expect("Only"))
            .collect()
    }

    fn non_critical_bricks<'a>(&'a self, critical: &'a HashSet<usize>) -> impl Iterator<Item=usize> + 'a {
        self.bricks.values()
            .flat_map(|v| v.iter().filter(|b| !critical.contains(&b.id)).map(|b| b.id))
            .unique()
    }

    fn simulate_unstable_bricks(&self) -> HashMap<usize, usize> {
        let critical = self.critical_bricks();
        if cfg!(debug_assertions) {
            // double-check that non-critical bricks don't need to be counted
            for brick in self.non_critical_bricks(&critical) {
                let mut tower = self.clone_without(brick);
                assert_eq!(tower.descend_all(), 0, "{} has unstable bricks", brick);
            }
        }

        let mut all_moved = HashMap::new();
        for brick in critical {
            let mut tower = self.clone_without(brick);
            all_moved.insert(brick, tower.descend_all());
        }
        all_moved
    }

    fn find_unstable_for(&self, brick: usize) -> HashSet<usize> {
        let mut removed = HashSet::from([brick]);
        // It might be more correct to use a priority queue ordered by z_top, but a bfs essentially
        // does that and seems to work as long as removed is populated at the same time the node is
        // added to the frontier
        let mut frontier = VecDeque::from([brick]);

        while let Some(cur) = frontier.pop_front() {
            if let Some(supports) = self.supports.get(&cur) {
                for supported in supports {
                    let supported_by = self.supported_by.get(supported).expect("Is supported");
                    if supported_by.iter().all(|b| removed.contains(b)) {
                        if removed.insert(*supported) {
                            frontier.push_back(*supported);
                        }
                    } else {
                    }
                }
            }
        }

        assert!(removed.remove(&brick)); // Initially removed brick doesn't count
        removed
    }

    fn find_all_unstable(&self) -> HashMap<usize, HashSet<usize>> {
        let critical = self.critical_bricks();
        if cfg!(debug_assertions) {
            // double-check that non-critical bricks don't need to be counted
            for brick in self.non_critical_bricks(&critical) {
                let unstable = self.find_unstable_for(brick);
                assert!(unstable.is_empty(), "{} has unstable bricks: {:?}", brick, unstable);
            }
        }
        let mut unstable = HashMap::new();
        for &brick in &critical {
            let for_brick = self.find_unstable_for(brick);
            assert!(!for_brick.is_empty());
            unstable.insert(brick, for_brick);
        }
        unstable
    }

    fn openscad(&self) -> String {
        let mut out = "module ocube(x1, y1, z1, x2, y2, z2) { translate([x1, y1, z1]) cube([x2-x1+1, y2-y1+1, z2-z1+1]); }\n\n".to_string();
        for brick in self.bricks.values().flatten() {
            let (x1, y1) = (brick.bounds2d.min.x, brick.bounds2d.min.y);
            let (x2, y2) = (brick.bounds2d.max.x, brick.bounds2d.max.y);
            out.push_str(&format!("ocube({},{},{}, {},{},{});\n", x1, y1, brick.z_top-brick.height+1, x2, y2, brick.z_top));
        }
        out
    }
}

#[derive(Debug, Hash, Eq, PartialEq, Copy, Clone)]
struct Brick {
    id: usize,
    bounds2d: Bounds2d,
    z_top: i32,
    height: i32,
}

impl Brick {
    fn create(id: usize, b: BrickStr) -> Brick {
        Brick{ id, bounds2d: b.bounds2d, z_top: b.z_top, height: b.height }
    }
}
#[derive(Debug)]
struct BrickStr {
    bounds2d: Bounds2d,
    z_top: i32,
    height: i32,
}

impl FromStr for BrickStr {
    type Err = Error;

    fn from_str(s: &str) -> Result<Self> {
        let (_, x1, y1, z1, x2, y2, z2) = regex_captures!(r"(\d+),(\d+),(\d+)~(\d+),(\d+),(\d+)", s)
            .with_context(|| format!("Invalid: {}", s))?;
        let a = point2d(x1.parse()?, y1.parse()?);
        let b = point2d(x2.parse()?, y2.parse()?);
        let (z1, z2): (i32, i32) = (z1.parse()?, z2.parse()?);
        let z_top = std::cmp::max(z1, z2);
        let height = (z2 - z1).abs() + 1;
        Ok(BrickStr{ bounds2d: Bounds2d::from_points(&[a, b]).context("Invalid")?, z_top, height })
    }
}

fn parse_input(input: &str) -> Result<Tower> {
    let bricks = input.lines().map(|l| l.parse()).collect::<Result<Vec<_>>>()?;
    Ok(Tower::create(bricks))
}

#[cfg(test)]
mod tests {
    use itertools::Itertools;
    use super::*;

    #[test]
    fn check_input() { parse_input(include_str!("input.txt")).unwrap(); }

    fn sorted_values(map: &HashMap<usize, Vec<usize>>) -> HashMap<usize, Vec<usize>> {
        map.iter()
            .map(|(k, v)| (*k, v.iter().cloned().sorted().collect()))
            .collect()
    }
    
    #[test]
    fn example() {
        let mut example = parse_input(include_str!("example.txt")).unwrap();
        let expected = parse_input(include_str!("example_settled.txt")).unwrap();

        example.descend_all();
        for i in example.bricks.keys().chain(expected.bricks.keys()).sorted().unique() {
            assert_eq!(example.bricks.get(i), expected.bricks.get(i), "{}", i);
        }
        assert_eq!(example.bricks, expected.bricks);

        let expected_supported_by = [
            (2, vec![1]), (3, vec![1]),
            (4, vec![2, 3]), (5, vec![2, 3]),
            (6, vec![4, 5]),
            (7, vec![6])
        ].into_iter().collect::<HashMap<usize, Vec<usize>>>();
        assert_eq!(sorted_values(&example.supported_by), expected_supported_by);

        let expected_supports = [
            (1, vec![2, 3]),
            (2, vec![4, 5]), (3, vec![4, 5]),
            (4, vec![6]), (5, vec![6]),
            (6, vec![7]),
        ].into_iter().collect::<HashMap<usize, Vec<usize>>>();
        assert_eq!(sorted_values(&example.supports), expected_supports);

        assert_eq!(example.critical_bricks(), [1, 6].into_iter().collect());

        let expected_unstable: HashMap<usize, HashSet<usize>> = [
            (1, HashSet::from([2, 3, 4, 5, 6, 7])),
            (6, HashSet::from([7])),
        ].into_iter().collect();
        let expected_unstable_lens: HashMap<usize, usize> = expected_unstable.iter()
            .map(|(k, v)| (*k, v.len())).collect();
        assert_eq!(example.simulate_unstable_bricks(), expected_unstable_lens);
        assert_eq!(example.find_all_unstable(), expected_unstable);
    }

    #[test]
    fn stable_above() {
        // bricks in a rectangle shape, one on bottom and one on top, with two pillars and two layers
        // removing the bottom should remove all seven, but removing a pillar should leave those above
        let bricks = [
            "1,0,1~1,2,1",
            "1,0,2~1,0,3",
            "1,2,2~1,2,3",
            "1,0,5~1,2,5",
            "1,0,6~1,0,6",
            "1,2,6~1,2,6",
            "1,0,7~1,2,7",
        ].into_iter().map(|b| b.parse()).collect::<Result<Vec<_>>>().unwrap();
        let mut tower = Tower::create(bricks);
        assert_eq!(tower.descend_all(), 4);

        assert_eq!(tower.critical_bricks(), [1, 4].into_iter().collect());

        let expected_unstable: HashMap<usize, HashSet<usize>> = [
            (1, HashSet::from([2, 3, 4, 5, 6, 7])),
            (4, HashSet::from([5, 6, 7])),
        ].into_iter().collect();
        let expected_unstable_lens: HashMap<usize, usize> = expected_unstable.iter()
            .map(|(k, v)| (*k, v.len())).collect();
        assert_eq!(tower.simulate_unstable_bricks(), expected_unstable_lens);
        assert_eq!(tower.find_all_unstable(), expected_unstable);
    }
}
