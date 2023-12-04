use std::collections::HashMap;
use std::str::FromStr;
use anyhow::*;
use lazy_regex::regex;

use advent_2023::euclid::{Point, point};

fn main() -> Result<()> {
    let schematic: Schematic = include_str!("input.txt").parse()?;
    println!("Part number sum: {}", schematic.valid_part_ids().iter().sum::<u32>());
    println!("Gear ratios product: {}", schematic.all_gears().values()
        .map(|v| v.iter().product::<u32>()).sum::<u32>());

    Ok(())
}

#[derive(Debug)]
struct Part {
    id: u32,
    min: Point,
    max: Point,
}

#[derive(Debug)]
struct Schematic {
    parts: Vec<Part>,
    symbols: HashMap<Point, char>,
}

impl Schematic {
    #[allow(dead_code)]
    fn valid_part_any_symbol(&self, part: &Part) -> bool {
        for y in part.min.y-1..=part.max.y+1 {
            for x in part.min.x-1..=part.max.x+1 {
                let p = point(x, y);
                if self.symbols.contains_key(&p) {
                    return true;
                }
            }
        }
        false
    }

    fn valid_part_each_symbol(&self, part: &Part) -> bool {
        for symbol in self.symbols.keys() {
            if symbol.in_bounds(point(part.min.x-1, part.min.y-1), point(part.max.x+1, part.max.y+1)) {
                return true;
            }
        }
        false
    }

    // Neither approach is really optimal, but using in_bounds() on each symbol benchmarks faster
    // than a linear search for nearby symbols even though it's O(n*m) vs. O(n)
    fn valid_part_ids(&self) -> Vec<u32> {
        self.parts.iter().filter(|p| self.valid_part_each_symbol(p)).map(|p| p.id).collect()
    }

    fn all_gears(&self) -> HashMap<Point, Vec<u32>> {
        let mut gears: HashMap<Point, Vec<u32>> = HashMap::new();
        for part in &self.parts {
            for y in part.min.y-1..=part.max.y+1 {
                for x in part.min.x - 1..=part.max.x + 1 {
                    let p = point(x, y);
                    if self.symbols.get(&p) == Some(&'*') {
                        gears.entry(p).and_modify(|v| v.push(part.id)).or_insert(vec!(part.id));
                    }
                }
            }
        }
        gears.into_iter().filter(|(_, v)| v.len() == 2).collect()
    }
}

impl FromStr for Schematic {
    type Err = Error;

    fn from_str(s: &str) -> Result<Self> {
        let number_re = regex!(r"\d+");
        let mut parts = Vec::new();
        let mut symbols = HashMap::new();
        for (y, line) in s.lines().enumerate() {
            let y = y as i32;
            for m in number_re.captures_iter(line).map(|c| c.get(0).expect("0-match")) {
                let part = Part{id: m.as_str().parse()?, min: point(m.start() as i32, y), max: point(m.end() as i32 - 1, y), };
                parts.push(part);
            }
            for (x, c) in line.chars().enumerate() {
                let x = x as i32;
                if c != '.' && !c.is_ascii_digit() {
                    symbols.insert(point(x, y), c);
                }
            }
        }
        Ok(Schematic{ parts, symbols })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn check_input() { include_str!("input.txt").parse::<Schematic>().unwrap(); }

    #[test]
    fn part_numbers() {
        let schematic: Schematic = include_str!("example.txt").parse().unwrap();
        assert_eq!(schematic.valid_part_ids(), [467, 35, 633, 617, 592, 755, 664, 598]);
    }

    #[test]
    fn gear_ratios() {
        let schematic: Schematic = include_str!("example.txt").parse().unwrap();
        let expected: HashMap<Point, Vec<u32>> = [((3, 1), [467, 35]), ((5, 8), [755, 598])].into_iter()
            .map(|(p, v)| (point(p.0, p.1), v.to_vec())).collect();
        assert_eq!(schematic.all_gears(), expected);
    }
}
