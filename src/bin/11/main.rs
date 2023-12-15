use std::collections::{BTreeMap, BTreeSet, HashSet};
use std::fmt::{Display, Formatter};
use std::ops::RangeInclusive;
use std::str::FromStr;
use anyhow::*;

use advent_2023::euclid::{Bounds, Point, point};

fn main() -> Result<()> {
    let input: StarChart = include_str!("input.txt").parse()?;
    let expanded = StarChart::expand_space(&input, 1);
    println!("Expanded distances (1): {}", expanded.pair_distances().values().sum::<u64>());
    let expanded = StarChart::expand_space(&input, 1000000-1);
    println!("Expanded distances (1000000): {}", expanded.pair_distances().values().sum::<u64>());


    Ok(())
}

#[derive(Debug, Eq, PartialEq)]
struct StarChart {
    // Yes they're galaxies but the term is "start chart" hence "stars"
    stars: HashSet<Point>,
    bounds: Bounds,
}

impl StarChart {
    fn create(stars: HashSet<Point>) -> StarChart {
        let bounds = Bounds::from_points(&stars).expect("Non-empty");
        StarChart{ stars, bounds }
    }

    fn empty_offsets(range: RangeInclusive<usize>, occupied: HashSet<usize>, expand_by: usize) -> Vec<usize> {
        range.map(|c| if occupied.contains(&c) { 0 } else { expand_by })
            .fold(Vec::new(), |mut v, c| {
                let next = v.last().map(|p| p + c).unwrap_or(c);
                v.push(next);
                v
            })
    }

    fn expand_space(&self, expand_by: usize) -> StarChart {
        assert_eq!(self.bounds.min, point(0, 0), "Casting to usize is not safe");
        let col_offsets: Vec<usize> = Self::empty_offsets(
            self.bounds.min.x as usize..=self.bounds.max.x as usize,
            self.stars.iter().map(|p| p.x as usize).collect(),
             expand_by);
        let row_offsets: Vec<usize> = Self::empty_offsets(
            self.bounds.min.y as usize..=self.bounds.max.y as usize,
            self.stars.iter().map(|p| p.y as usize).collect(),
            expand_by);

        let mut expanded = HashSet::new();
        for star in &self.stars {
            expanded.insert(point(
                star.x + (col_offsets[star.x as usize] as i32),
                star.y + (row_offsets[star.y as usize] as i32)));
        }
        StarChart::create(expanded)
    }

    fn pair_distances(&self) -> BTreeMap<(Point, Point), u64> {
        let mut stars: BTreeSet<_> = self.stars.iter().cloned().collect();
        let mut pairs = BTreeMap::new();
        while let Some(star) = stars.pop_first() {
            for &other in &stars {
                pairs.insert((star, other), (star - other).grid_len() as u64);
            }
        }
        pairs
    }
}

impl Display for StarChart {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        let mut out = String::new();
        for row in self.bounds.iter_rows() {
            for pos in row {
                match self.stars.contains(&pos) {
                    true => out.push('#'),
                    false => out.push('.'),
                }
            }
            out.push('\n');
        }
        assert_eq!(out.pop(), Some('\n')); // removed trailing newline
        write!(f, "{}", out)
    }
}

impl FromStr for StarChart {
    type Err = Error;

    fn from_str(s: &str) -> Result<Self> {
        let mut stars = HashSet::new();
        for (y, l) in s.lines().enumerate() {
            for (x, c) in l.chars().enumerate() {
                if c == '#' {
                    let pos = point(x as i32, y as i32);
                    stars.insert(pos);
                }
            }
        }
        Ok(StarChart::create(stars))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn check_input() { include_str!("input.txt").parse::<StarChart>().unwrap(); }

    #[test]
    fn example() {
        let star_chart: StarChart = include_str!("example.txt").parse().unwrap();
        let expanded = star_chart.expand_space(1);
        let expected: StarChart = include_str!("example_expanded.txt").parse().unwrap();
        assert_eq!(expanded, expected);
        let distances = expanded.pair_distances();
        distances.iter().for_each(|e| println!("{:?}", e));
        assert_eq!(distances[&(point(1,6), point(5, 11))], 9);
        assert_eq!(distances[&(point(4,0), point(9, 10))], 15);
        assert_eq!(distances[&(point(0,2), point(12, 7))], 17);
        assert_eq!(distances[&(point(0,11), point(5, 11))], 5);
        assert_eq!(distances.values().sum::<u64>(), 374, "{:?}", expanded);
    }

    // Spoilers related to cache index
    parameterized_test::create!{ expand, (n, dist), {
        let star_chart: StarChart = include_str!("example.txt").parse().unwrap();
        let expanded = star_chart.expand_space(n-1);
        assert_eq!(expanded.pair_distances().values().sum::<u64>(), dist);
    } }
    expand! {
        // See also https://old.reddit.com/r/adventofcode/comments/18fx0to/ for closed-form solution
        e1:       (1, 292),            // not specified
        e10:      (10, 1030),
        e100:     (100, 8410),
        e1000:    (1000, 82210),       // not specified
        e10000:   (10000, 820210),     // not specified
        e100000:  (100000, 8200210),   // not specified
        e1000000: (1000000, 82000210), // not specified
    }
}
