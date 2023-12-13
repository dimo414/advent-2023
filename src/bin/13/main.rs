use std::collections::HashSet;
use std::str::FromStr;
use anyhow::*;

use advent_2023::elapsed;
use advent_2023::euclid::{Bounds, Point, point};

fn main() -> Result<()> {
    let input = parse_input(include_str!("input.txt"))?;

    let sum = elapsed!("Part 1", score(&input, 0));
    println!("Mirror score: {}", sum);
    let sum = elapsed!("Part 2", score(&input, 1));
    println!("Smudged score: {}", sum);

    Ok(())
}

fn score(landscapes: &[Landscape], expected_errors: u32) -> i32 {
    landscapes.iter().enumerate().map(|(i, l)| {
        if let Some(x_edge) = l.horizontal_reflection(expected_errors) {
            debug_assert_eq!(l.vertical_reflection(expected_errors), None);
            return x_edge;
        }
        if let Some(y_edge) = l.vertical_reflection(expected_errors) {
            return 100 * y_edge;
        }
        panic!("No valid mirror found for item #{}", i);
    }).sum()
}

#[derive(Debug)]
struct Landscape {
    rocks: HashSet<Point>,
    bounds: Bounds,
}

impl Landscape {
    fn horizontal_reflection(&self, expected_errors: u32) -> Option<i32> {
        for edge in self.bounds.min.x+1..=self.bounds.max.x {
            let mut errors = 0;
            for y in self.bounds.min.y..=self.bounds.max.y {
                errors += self.mirrored_row(edge, y, expected_errors - errors);
                if errors > expected_errors { break; }
            }
            if errors == expected_errors { return Some(edge); }
        }
        None
    }

    fn mirrored_row(&self, edge: i32, y: i32, max_errors: u32) -> u32 {
        let dist = std::cmp::min(edge-self.bounds.min.x, self.bounds.max.x+1-edge);
        assert_ne!(dist, 0);
        let mut errors = 0;
        for x_offset in 0..dist {
            let left = point(edge-1-x_offset, y);
            let right = point(edge+x_offset, y);
            if self.rocks.contains(&left) != self.rocks.contains(&right) {
                errors += 1;
                if errors > max_errors { break; }
            }
        }
        errors
    }

    fn vertical_reflection(&self, expected_errors: u32) -> Option<i32> {
        for edge in self.bounds.min.y+1..=self.bounds.max.y {
            let mut errors = 0;
            for x in self.bounds.min.x..=self.bounds.max.x {
                errors += self.mirrored_column(edge, x, expected_errors - errors);
                if errors > expected_errors { break; }
            }
            if errors == expected_errors { return Some(edge); }
        }
        None
    }

    fn mirrored_column(&self, edge: i32, x: i32, max_errors: u32) -> u32 {
        let dist = std::cmp::min(edge-self.bounds.min.y, self.bounds.max.y+1-edge);
        assert_ne!(dist, 0);
        let mut errors = 0;
        for y_offset in 0..dist {
            let top = point(x, edge-1-y_offset);
            let bottom = point(x, edge+y_offset);
            if self.rocks.contains(&top) != self.rocks.contains(&bottom) {
                errors += 1;
                if errors > max_errors { break; }
            }
        }
        errors
    }
}

impl FromStr for Landscape {
    type Err = Error;

    fn from_str(s: &str) -> Result<Self> {
        let mut rocks = HashSet::new();
        for (y, l) in s.lines().enumerate() {
            for (x, c) in l.chars().enumerate() {
                match c {
                    '#' => { rocks.insert(point(x as i32, y as i32)); },
                    '.' => {},
                    _ => bail!("Invalid"),
                }
            }
        }
        let bounds = Bounds::from_points(&rocks).context("Empty")?;
        Ok(Landscape{ rocks, bounds })
    }
}

fn parse_input(input: &str) -> Result<Vec<Landscape>> {
    input.split("\n\n").map(|s| s.parse()).collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn check_input() { parse_input(include_str!("input.txt")).unwrap(); }

    #[test]
    fn part1() {
        let example = parse_input(include_str!("example.txt")).unwrap();

        let horizontal: Vec<_> = example.iter().map(|l| l.horizontal_reflection(0)).collect();
        assert_eq!(horizontal, [Some(5), None]);
        let vertical: Vec<_> = example.iter().map(|l| l.vertical_reflection(0)).collect();
        assert_eq!(vertical, [None, Some(4)]);

        assert_eq!(score(&example, 0), 405);
    }

    #[test]
    fn part2() {
        let example = parse_input(include_str!("example.txt")).unwrap();

        let horizontal: Vec<_> = example.iter().map(|l| l.horizontal_reflection(1)).collect();
        assert_eq!(horizontal, [None, None]);
        let vertical: Vec<_> = example.iter().map(|l| l.vertical_reflection(1)).collect();
        assert_eq!(vertical, [Some(3), Some(1)]);

        assert_eq!(score(&example, 1), 400);
    }
}
