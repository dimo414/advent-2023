use std::collections::HashMap;
use anyhow::*;
use itertools::Itertools;
use lazy_regex::regex_captures;

fn main() -> Result<()> {
    let (dirs, paths) = parse_input(include_str!("input.txt"))?;
    let (dest, dist) = steps_to(&dirs, &paths, "AAA");
    println!("Steps to {}: {}", dest, dist);

    let mut steps = Vec::new();
    for source in all_starts(&paths) {
        let (_dest, dist) = steps_to(&dirs, &paths, source);
        //println!("\tSteps to {}: {}", _dest, dist);
        steps.push(dist);
    }
    println!("Steps to all ..Z's: {}", fold_lcm(&steps));

    Ok(())
}

fn steps_to<'a>(dirs: &str, paths: &'a HashMap<String, (String, String)>, start: &'a str) -> (&'a str, u64) {
    let mut cur = start;
    let mut steps = 0;
    for dir in dirs.chars().cycle() {
        let (left, right) = paths.get(cur).expect("Not in map");
        cur = match dir {
            'L' => left,
            'R' => right,
            _ => panic!(),
        };
        steps += 1;
        if cur.ends_with('Z') {
            return (cur, steps)
        }
    }
    unreachable!()
}

fn all_starts(paths: &HashMap<String, (String, String)>) -> impl Iterator<Item=&String> {
    paths.keys().filter(|p| p.ends_with('A'))
}

fn fold_lcm<'a>(inputs: impl IntoIterator<Item=&'a u64>) -> u64 {
    inputs.into_iter().fold(1, |lcm,&v| num::integer::lcm(lcm, v))
}

#[cfg(test)]
fn steps_to_all(dirs: &str, paths: &HashMap<String, (String, String)>) -> u64 {
    let mut curs: Vec<_> = all_starts(paths).collect();
    let n = curs.len();
    let mut steps = 0;
    for dir in dirs.chars().cycle() {
        for item in curs.iter_mut().take(n) {
            let (left, right) = paths.get(*item).expect("Not in map");
            let mut next = match dir {
                'L' => left,
                'R' => right,
                _ => panic!(),
            };
            std::mem::swap(item, &mut next);
        }
        steps += 1;
        if curs.iter().all(|n| n.ends_with('Z')) {
            return steps;
        }
    }
    unreachable!()
}

fn parse_input(input: &str) -> Result<(String, HashMap<String, (String, String)>)> {
    let (dirs, paths) = input.split("\n\n").collect_tuple().context("Invalid")?;
    let paths = paths.lines().map(|l| {
        let (_, cur, left, right) = regex_captures!(r"([^ ]+) = \(([^ ]+), ([^ ]+)\)", l).with_context(|| format!("Invalid: {}", l))?;
        Ok((cur.to_string(), (left.to_string(), right.to_string())))
    }).collect::<Result<HashMap<_, _>>>()?;
    Ok((dirs.to_string(), paths))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn check_input() { parse_input(include_str!("input.txt")).unwrap(); }

    parameterized_test::create!{ part1, (input, expected), {
        let (dirs, paths) = parse_input(input).unwrap();
        assert_eq!(steps_to(&dirs, &paths, "AAA"), expected);
    } }
    part1! {
        example1: (include_str!("example1.txt"), ("ZZZ", 2)),
        example2: (include_str!("example2.txt"), ("ZZZ", 6)),
    }

    #[test]
    fn part2_brute_force() {
        let (dirs, paths) = parse_input(include_str!("example3.txt")).unwrap();
        assert_eq!(steps_to_all(&dirs, &paths), 6);
    }

    #[test]
    fn part2() {
        let (dirs, paths) = parse_input(include_str!("example3.txt")).unwrap();
        let starts: Vec<_> = all_starts(&paths).sorted().collect();
        assert_eq!(starts, ["11A", "22A"]);
        let all_steps: Vec<_> = starts.iter().map(|start| steps_to(&dirs, &paths, start)).collect();
        assert_eq!(all_steps, [("11Z", 2), ("22Z", 3)]);
        assert_eq!(fold_lcm(all_steps.iter().map(|(_, s)|s)), 6);
    }
}
