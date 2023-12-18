use std::collections::{HashSet, VecDeque};
use std::str::FromStr;
use anyhow::*;
use lazy_regex::regex_captures;
use advent_2023::elapsed;

use advent_2023::euclid::{Bounds, bounds, Point, Vector, vector};

fn main() -> Result<()> {
    let input = parse_input(include_str!("input.txt"))?;
    if cfg!(feature="timing") {
        let mut lagoon = elapsed!("Border", Lagoon::create(&input));
        println!("Perimeter: {}", lagoon.border.len());
        println!("Area: {}", elapsed!("Interior", lagoon.trench_interior()) + lagoon.border.len() as i32);
    }
    println!("Initial Area: {}", elapsed!("Polygon", Polygon::create(input.iter().map(|i| &i.path)).area()));
    println!("Corrected Area: {}", elapsed!("Color Polygon", Polygon::create(input.iter().map(|i| &i.color_path)).area()));

    Ok(())
}

#[derive(Debug, Eq, PartialEq)]
struct Trench {
    path: Vector,
    color_path: Vector,
}

impl FromStr for Trench {
    type Err = Error;

    fn from_str(s: &str) -> Result<Self> {
        let (_, dir, dist, color_dist, color_dir) =
            regex_captures!(r"(.) (\d+) \(#(.{5})(.)\)", s).with_context(|| format!("Invalid: {}", s))?;
        let path = match dir {
            "U" => vector(0, -1),
            "D" => vector(0, 1),
            "L" => vector(-1, 0),
            "R" => vector(1, 0),
            _ => bail!("Invalid"),
        } * dist.parse::<i32>()?;

        let color_path = match color_dir {
            "0" => vector(1, 0),
            "1" => vector(0, 1),
            "2" => vector(-1, 0),
            "3" => vector(0, -1),
            _ => bail!("Invalid"),
        } * i32::from_str_radix(color_dist, 16)?;

        Ok(Trench{ path, color_path })
    }
}

#[derive(Debug)]
struct Lagoon {
    bounds: Bounds,
    border: HashSet<Point>,
}

impl Lagoon {
    fn create(instructions: &[Trench]) -> Lagoon {
        assert!(!instructions.is_empty()); // Use bail!() if this swaps to returning a result
        let mut border = HashSet::new();
        let mut pos = Point::ORIGIN;
        for trench in instructions {
            let dest = pos + trench.path;
            let path = Bounds::from_points(&[pos, dest]).expect("Non-empty");
            border.extend(path.iter());
            pos = dest;
        }
        let bounds = Bounds::from_points(&border).expect("Non-empty");
        Lagoon{ bounds, border }
    }

    fn trench_interior(&mut self) -> i32 {
        // Expand bounds by 1 so we know the flood fill will hit all exterior points
        let bounds = bounds(self.bounds.min + vector(-1,-1), self.bounds.max + vector(1,1));
        let mut queue: VecDeque<_> = [bounds.min].into();
        let mut outside: HashSet<_> = queue.iter().cloned().collect();
        while let Some(pos) = queue.pop_front() {
            for p in Vector::CARDINAL.iter().map(|d| pos + d) {
                if bounds.contains(p) && !outside.contains(&p) && !self.border.contains(&p) {
                    outside.insert(p);
                    queue.push_back(p);
                }
            }
        }
        bounds.area() - outside.len() as i32 - self.border.len() as i32
    }
}

struct Polygon {
    edges: Vec<(Point, Point)>,
    perimeter: i64,
}

impl Polygon {
    fn create<'a>(paths: impl IntoIterator<Item = &'a Vector>) -> Polygon {
        let mut edges = Vec::new();
        let mut perimeter = 0;
        let mut pos = Point::ORIGIN;
        for path in paths.into_iter() {
            perimeter += path.grid_len() as i64;
            let dest = pos + path;
            edges.push((pos, dest));
            pos = dest;
        }
        debug_assert_eq!(edges.last().expect("Non-empty").1, Point::ORIGIN);
        Polygon{ edges, perimeter }
    }

    // https://en.wikipedia.org/wiki/Shoelace_formula#Trapezoid_formula
    fn shoelace_area(&self) -> i64 {
        self.edges.iter()
            .map(|e| (e.0.y + e.1.y + 1) as i64 * (e.0.x - e.1.x) as i64)
            .sum::<i64>().abs() / 2
    }

    fn area(&self) -> i64 {
        // Intuitively, we need to add ~half the perimeter to our area to account for the inclusive
        // nature of our points. For example a box out to (4, 4) has an area of 25, not 16 - adding
        // ~half the 4x4 perimeter (4 + 4 for the edges + 1 for the corner) gets the correct area.
        // It's less-obvious that this is correct in general (e.g. for shapes with overlapping
        // perimeters) but it at least works for the given input.
        //
        // That said, I'm still not sold that this would work for any legal input; trenches that run
        // parallel or crisscross repeatedly might mess things up. Notably, this approach aligns
        // with https://en.wikipedia.org/wiki/Pick%27s_theorem but supposedly additional information
        // such as winding numbers are needed for polygons that can cross themselves.
        self.shoelace_area() + self.perimeter/2 + 1
    }
}

fn parse_input(input: &str) -> Result<Vec<Trench>> {
    input.lines().map(|l| l.parse()).collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn check_input() { parse_input(include_str!("input.txt")).unwrap(); }

    #[test]
    fn parse_example() {
        let example = parse_input(include_str!("example.txt")).unwrap();
        assert_eq!(example, [
            Trench { path: vector(6, 0),  color_path: vector(461937, 0) },
            Trench { path: vector(0, 5),  color_path: vector(0, 56407) },
            Trench { path: vector(-2, 0), color_path: vector(356671, 0) },
            Trench { path: vector(0, 2),  color_path: vector(0, 863240) },
            Trench { path: vector(2, 0),  color_path: vector(367720, 0) },
            Trench { path: vector(0, 2),  color_path: vector(0, 266681) },
            Trench { path: vector(-5, 0), color_path: vector(-577262, 0) },
            Trench { path: vector(0, -2), color_path: vector(0, -829975) },
            Trench { path: vector(-1, 0), color_path: vector(-112010, 0) },
            Trench { path: vector(0, -2), color_path: vector(0, 829975) },
            Trench { path: vector(2, 0),  color_path: vector(-491645, 0) },
            Trench { path: vector(0, -3), color_path: vector(0, -686074) },
            Trench { path: vector(-2, 0), color_path: vector(-5411, 0) },
            Trench { path: vector(0, -2), color_path: vector(0, -500254) },
        ]);
    }

    #[test]
    fn lagoon() {
        let example = parse_input(include_str!("example.txt")).unwrap();
        let mut lagoon = Lagoon::create(&example);
        assert_eq!(lagoon.border.len(), 38);
        assert_eq!(lagoon.trench_interior() + lagoon.border.len() as i32, 62);
    }

    #[test]
    fn polygon() {
        let example = parse_input(include_str!("example.txt")).unwrap();
        let polygon = Polygon::create(example.iter().map(|i| &i.path));
        assert_eq!(polygon.perimeter, 38);
        assert_eq!(polygon.area(), 62);
    }

    #[test]
    fn polygon_color() {
        let example = parse_input(include_str!("example.txt")).unwrap();
        let polygon = Polygon::create(example.iter().map(|i| &i.color_path));
        assert_eq!(polygon.perimeter, 6405262); // Not specified
        assert_eq!(polygon.area(), 952408144115);
    }
}
