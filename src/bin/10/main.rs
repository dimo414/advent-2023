use std::collections::{HashMap, HashSet, VecDeque};
use std::fmt::{Debug, Display, Formatter};
use std::str::FromStr;
use std::time::Instant;
use anyhow::*;

use advent_2023::collect::MoreItertools;
use advent_2023::euclid::{Bounds, Point, point, Vector, vector};
use advent_2023::pathfinding::{Edge, Graph};
use advent_2023::terminal::{Color, Terminal, TerminalDisplay, TerminalString};

fn main() -> Result<()> {
    let _drop = Terminal::init();

    let map: Map = include_str!("example4.txt").parse()?;
    let members = map.loop_members();
    println!("Members: {}", members.len() / 2);
    println!("Interior: {:?}", map.interior(&members).len());
    Terminal::interactive_color_display(&map, Instant::now());

    Ok(())
}

#[derive(Debug, Copy, Clone, Eq, PartialEq)]
enum Pipe {
    Vertical,
    Horizontal,
    NorthEast,
    NorthWest,
    SouthWest,
    SouthEast,
    Start,
}

impl Pipe {
    fn directions(&self) -> [Vector; 2] {
        use Pipe::*;
        match self {
            Vertical => [vector(0, -1), vector(0, 1)],
            Horizontal => [vector(-1, 0), vector(1, 0)],
            NorthEast=> [vector(0, -1), vector(1, 0)],
            NorthWest=> [vector(0, -1), vector(-1, 0)],
            SouthWest=> [vector(0, 1), vector(-1, 0)],
            SouthEast=> [vector(0, 1), vector(1, 0)],
            Start=> panic!(),
        }
    }

    fn to_char(&self) -> char {
        use Pipe::*;
        match self {
            Vertical => '║',
            Horizontal => '═',
            NorthEast=> '╚',
            NorthWest=> '╝',
            SouthWest=> '╗',
            SouthEast=> '╔',
            Start=> '╬',
        }
    }
}

impl TryFrom<char> for Pipe {
    type Error = Error;

    fn try_from(value: char) -> Result<Self> {
        use Pipe::*;
        Ok(match value {
            '|' => Vertical,
            '-' => Horizontal,
            'L' => NorthEast,
            'J' => NorthWest,
            '7' => SouthWest,
            'F' => SouthEast,
            'S' => Start,
            _ => bail!("invalid: {:?}", value),
        })
    }
}

impl Display for Pipe {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.to_char())
    }
}

impl FromStr for Pipe {
    type Err = Error;

    fn from_str(s: &str) -> Result<Self> {
        s.chars().drain_only()?.try_into()
    }
}

#[derive(Debug)]
struct Map {
    pipes: HashMap<Point, Pipe>,
    bounds: Bounds,
    start: Point,
}

impl Map {
    fn start_type(&self) -> Pipe {
        debug_assert_eq!(self.pipes.get(&self.start), Some(&Pipe::Start));
        let mut neighbors = Vec::new();
        for dir in Vector::CARDINAL.iter() {
            let neighbor = self.start + dir;
            if let Some(rev_dirs) = self.pipes.get(&neighbor).map(|p| p.directions()) {
                if rev_dirs.iter().any(|d| neighbor + d == self.start) {
                    neighbors.push(*dir);
                }
            }
        }
        neighbors.sort_by_key(|v| v.x); // Incorrect but good enough for this task
        assert_eq!(neighbors.len(), 2);
        match neighbors[..] {
            [Vector{x:-1, y:0}, Vector{x:0, y:-1}] => { Pipe::NorthWest },
            [Vector{x:0, y:1}, Vector{x:1, y:0}] => { Pipe::SouthEast },
            _ => panic!("{:?}", neighbors),
        }
    }

    #[cfg(test)]
    fn loop_distance(&self) -> usize {
        let bfs_routes = self.bfs_all(&self.start);
        bfs_routes.iter().map(|(_dest, route)| route.len()).max().expect("Non-empty") - 1
    }

    fn loop_members(&self) -> HashSet<Point> {
        let mut seen = HashSet::new();
        let mut frontier = VecDeque::new();
        frontier.push_back(self.start);
        while let Some(current) = frontier.pop_front() {
            seen.insert(current);
            for edge in self.neighbors(&current) {
                let next = edge.dest();
                if !seen.contains(next) {
                    frontier.push_back(*next);
                }
            }
        }
        seen
    }

    fn interior(&self, loop_members: &HashSet<Point>) -> HashSet<Point> {
        let mut interior = HashSet::new();
        for row in self.bounds.iter_rows() {
            let mut crossed = 0;
            let mut last_bend = None;
            for pos in row {
                if loop_members.contains(&pos) {
                    use Pipe::*;
                    let mut pipe = *self.pipes.get(&pos).expect("Present");
                    if pipe == Pipe::Start {
                        pipe = self.start_type();
                    }
                    match pipe {
                        Horizontal => {},
                        NorthEast|SouthEast => {
                            assert_eq!(last_bend, None);
                            last_bend = Some(pipe);
                        },
                        NorthWest|SouthWest => {
                            match (pipe, last_bend.expect("Must be Some")) {
                                (NorthWest, SouthEast)|(SouthWest, NorthEast) => { crossed += 1; },
                                (NorthWest, NorthEast)|(SouthWest, SouthEast) => {},
                                _ => panic!(),
                            }
                            last_bend = None;
                        }
                        Vertical => { crossed += 1; },
                        Start => { unreachable!(); },
                    }
                } else if crossed % 2 == 1 {
                    interior.insert(pos);
                }
            }
        }
        interior
    }
}

impl Graph for Map {
    type Node = Point;

    fn neighbors(&self, source: &Self::Node) -> Vec<Edge<Self::Node>> {
        let mut cur_pipe = *self.pipes.get(source).expect("Missing");
        if cur_pipe == Pipe::Start {
            cur_pipe = self.start_type();
        }
        cur_pipe.directions().iter().map(|dest| Edge::new(1, *source, source+dest)).collect()
    }
}

impl Display for Map {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        let mut out = String::new();
        for row in self.bounds.iter_rows() {
            for pos in row {
                match self.pipes.get(&pos) {
                    Some(pipe) => out.push_str(&pipe.to_string()),
                    None => out.push('.'),
                }
            }
            out.push('\n');
        }
        assert_eq!(out.pop(), Some('\n')); // removed trailing newline
        write!(f, "{}", out)
    }
}

impl TerminalDisplay for Map {
    fn colored_display(&self, width_hint: usize, height_hint: usize) -> TerminalString {
        // TODO consider if there's a reasonable way to cache these calls instead of recomputing them
        let members = self.loop_members();
        let interior = self.interior(&members);

        let mut pixels = Vec::new();
        for row in self.bounds.iter_rows().take(height_hint) {
            for pos in row.take(width_hint) {
                let c = self.pipes.get(&pos).map_or('.', |p| p.to_char());
                let color = if self.start == pos {
                    Some(Color::YELLOW)
                } else if members.contains(&pos) {
                    Some(Color::GREEN)
                } else if interior.contains(&pos) {
                    // TODO it'd be much better to use a background color here
                    Some(Color::BLUE)
                } else {
                    None
                };
                pixels.push((c, color))
            }
        }

        TerminalString{ pixels, width: (self.bounds.max.x+1-self.bounds.min.x) as usize }
    }
}

impl FromStr for Map {
    type Err = Error;

    fn from_str(s: &str) -> Result<Self> {
        let mut pipes = HashMap::new();
        let mut start = None;
        for (y, l) in s.lines().enumerate() {
            for (x, c) in l.chars().enumerate() {
                if c != '.' {
                    let pos = point(x as i32, y as i32);
                    let pipe = c.try_into()?;
                    pipes.insert(pos, pipe);
                    if let Pipe::Start = pipe {
                        assert!(start.is_none());
                        start = Some(pos);
                    }
                }
            }
        }
        let bounds = Bounds::from_points(pipes.keys()).context("Non-empty")?;
        let start = start.context("Start position not found")?;
        Ok(Map{ pipes, bounds, start })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn check_input() { include_str!("input.txt").parse::<Map>().unwrap(); }

    parameterized_test::create!{ pipes, (s, dist), {
        let map = s.parse::<Map>().unwrap();
        let pipes = map.loop_members();
        assert_eq!(pipes.len(), dist*2);
        // Check BFS approach works
        assert_eq!(map.loop_distance(), dist);
    }}
    pipes! {
        e1: (include_str!("example1.txt"), 4),
        e2: (include_str!("example2.txt"), 8),
        e3: (include_str!("example3.txt"), 23), // unspecified
        e4: (include_str!("example4.txt"), 70), // unspecified
    }

    parameterized_test::create!{ interior, (s, expected), {
        let map = s.parse::<Map>().unwrap();
        let members = map.loop_members();
        assert_eq!(map.interior(&members).len(), expected);
    }}
    interior! {
        e1: (include_str!("example1.txt"), 1),
        e2: (include_str!("example2.txt"), 1),
        e3: (include_str!("example3.txt"), 4),
        e4: (include_str!("example4.txt"), 8),
    }
}
