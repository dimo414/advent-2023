use std::collections::HashMap;
use std::fmt::{Display, Formatter};
use std::str::FromStr;
use anyhow::*;

use advent_2023::elapsed;
use advent_2023::euclid::{Bounds, Point, point, vector, Vector};

fn main() -> Result<()> {
    let mut input: Platform = include_str!("input.txt").parse()?;
    input.tip(vector(0, -1));
    println!("Initial load: {}", input.north_load());
    // This re-does one tip((0, -1)) but it's a no-op so it's harmless aside from the CPU time
    // It so happens that a lookback of 1 is sufficient for the input, but the example requires at
    // least 3 lookback due to duplicate numbers prior to the cycle start, so use 5 to be safe.
    println!("Long-term load: {}", elapsed!(input.load_after(5, 1000000000)));

    Ok(())
}

#[derive(Debug, Eq, PartialEq)]
enum Rock {
    Round,
    Cube,
}

#[derive(Debug, Eq, PartialEq)]
struct Platform {
    grid: HashMap<Point, Rock>,
    bounds: Bounds,
}

impl Platform {
    fn tip(&mut self, dir: Vector) {
        fn iter_order(bounds: Bounds, dir: Vector) -> Box<dyn Iterator<Item=Point>> {
            // This is a bit gross, could probably be avoided
            match dir {
                Vector{x:0, y:-1} => Box::new((bounds.min.y..=bounds.max.y)
                    .flat_map(move |y| (bounds.min.x..=bounds.max.x).map(move |x| point(x, y)))),
                Vector{x:0, y:1} => Box::new((bounds.min.y..=bounds.max.y).rev()
                    .flat_map(move |y| (bounds.min.x..=bounds.max.x).map(move |x| point(x, y)))),
                Vector{x:-1, y:0} => Box::new((bounds.min.x..=bounds.max.x)
                    .flat_map(move |x| (bounds.min.y..=bounds.max.y).map(move |y| point(x, y)))),
                Vector{x:1, y:0} => Box::new((bounds.min.x..=bounds.max.x).rev()
                    .flat_map(move |x| (bounds.min.y..=bounds.max.y).map(move |y| point(x, y)))),
                _ => unimplemented!(),
            }
        }

        for pos in iter_order(self.bounds, dir) {
            if !matches!(self.grid.get(&pos), Some(Rock::Round)) { continue; }
            let mut dest = pos;
            loop {
                let next = dest + dir;
                if !self.bounds.contains(next) || self.grid.contains_key(&next) {
                    break;
                }
                dest = next;
            }
            let value = self.grid.remove(&pos).expect("Must be present");
            self.grid.insert(dest, value);
        }
    }

    fn cycle(&mut self) {
        self.tip(vector(0, -1));
        self.tip(vector(-1, 0));
        self.tip(vector(0, 1));
        self.tip(vector(1, 0));
    }

    fn find_loop(&mut self, lookback: usize) -> (usize, Vec<u64>) {
        let mut loads = vec![self.north_load()];
        for _ in 0..=lookback {
            self.cycle();
            loads.push(self.north_load());
        }
        loop {
            self.cycle();
            loads.push(self.north_load());
            let tail = &loads[loads.len()-lookback..];
            for i in 0..(loads.len()-lookback-1) {
                if &loads[i..i+lookback] == tail {
                    return (i, loads[i..loads.len()-lookback].to_vec());
                }
            }
        }
    }

    fn north_load(&self) -> u64 {
        self.grid.iter()
            .filter(|(_, v)| matches!(v, Rock::Round))
            .map(|(p,_)| (self.bounds.max.y+1-p.y) as u64)
            .sum()
    }

    fn load_after(&mut self, lookback: usize, cycles: usize) -> u64 {
        let (offset, cycle) = self.find_loop(lookback);
        cycle[(cycles - offset) % cycle.len()]
    }
}

impl FromStr for Platform {
    type Err = Error;

    fn from_str(s: &str) -> Result<Self> {
        let mut grid = HashMap::new();
        for (y, l) in s.lines().enumerate() {
            for (x, c) in l.chars().enumerate() {
                let pos = point(x as i32, y as i32);
                match c {
                    'O' => { grid.insert(pos, Rock::Round); },
                    '#' => { grid.insert(pos, Rock::Cube); },
                    '.' => {},
                    _ => bail!("Invalid: {}", c),
                }
            }
        }
        let bounds = Bounds::from_points(grid.keys()).context("Invalid")?;
        Ok(Platform{ grid, bounds })
    }
}

impl Display for Platform {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        let mut out = String::new();
        for row in self.bounds.iter_rows() {
            for pos in row {
                match self.grid.get(&pos) {
                    Some(Rock::Round) => out.push('O'),
                    Some(Rock::Cube) => out.push('#'),
                    None => out.push('.'),
                }
            }
            out.push('\n');
        }
        assert_eq!(out.pop(), Some('\n'));
        write!(f, "{}", out)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn check_input() { include_str!("input.txt").parse::<Platform>().unwrap(); }

    #[test]
    fn tip_north() {
        let mut platform = include_str!("example.txt").parse::<Platform>().unwrap();
        platform.tip(vector(0, -1));
        assert_eq!(platform, include_str!("example_tipped.txt").parse().unwrap());
    }

    #[test]
    fn cycles() {
        let mut platform = include_str!("example.txt").parse::<Platform>().unwrap();
        platform.cycle();
        assert_eq!(platform, include_str!("example_c1.txt").parse().unwrap());
        platform.cycle();
        assert_eq!(platform, include_str!("example_c2.txt").parse().unwrap());
        platform.cycle();
        assert_eq!(platform, include_str!("example_c3.txt").parse().unwrap());
    }

    #[test]
    fn load_test() {
        let mut platform = include_str!("example.txt").parse::<Platform>().unwrap();
        assert_ne!(platform.load_after(1, 1000000000), 64);
        let mut platform = include_str!("example.txt").parse::<Platform>().unwrap();
        assert_ne!(platform.load_after(2, 1000000000), 64);
        let mut platform = include_str!("example.txt").parse::<Platform>().unwrap();
        assert_eq!(platform.load_after(3, 1000000000), 64);
        let mut platform = include_str!("example.txt").parse::<Platform>().unwrap();
        assert_eq!(platform.load_after(4, 1000000000), 64);
        let mut platform = include_str!("example.txt").parse::<Platform>().unwrap();
        assert_eq!(platform.load_after(5, 1000000000), 64);
    }
}
