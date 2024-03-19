use std::cmp::Ordering;
use std::collections::{HashMap, VecDeque};
use std::fmt::{Debug, Formatter};
use std::str::FromStr;
use anyhow::*;
use itertools::Itertools;
use lazy_regex::regex_captures;
use advent_2023::collect;

use advent_2023::collect::{MoreIntoIterator, Range};

const FULL_RANGE: Range = Range::create(1, 4001);

fn main() -> Result<()> {
    let (workflows, parts) = parse_input(include_str!("input.txt"))?;
    let score =  parts.iter().filter(|p| workflows.validate(p)).map(|p| p.score()).sum::<i32>();
    println!("Valid Parts Score: {}", score);

    println!("Count Valid Parts: {}", workflows.count_valid());
    Ok(())
}

struct Test {
    text: String,
    var: char,
    range: Range,
    dest: String,
}

impl Test {
    fn test_part(&self, part: &Part) -> bool {
        self.range.contains(part.var(self.var) as i64)
    }

    fn split_range(&self, parts: PartRange) -> (Option<PartRange>, Option<PartRange>) {
        let part_range = parts.var_range(self.var);
        let (mut pass, mut fail) = (None, None);
        if let Some(pass_range) = part_range.intersect(self.range) {
            pass = Some(parts.constrain(self.var, pass_range));
        }
        match part_range.difference(self.range) {
            collect::Difference::None => {},
            collect::Difference::One(fail_range) => {
                fail = Some(parts.constrain(self.var, fail_range));
            },
            collect::Difference::Two(one, two) => {
                panic!("Unexpected split, {:?}-{:?} = ({:?},{:?})", part_range, self.range, one, two);
            }
        }

        (pass, fail)
    }
}

impl FromStr for Test {
    type Err = Error;

    fn from_str(text: &str) -> std::result::Result<Self, Self::Err> {
        let (_, letter, cond, value, dest) = regex_captures!(r"([xmas])([<>])(\d+):(.+)", &text).with_context(|| format!("Invalid: {}", text))?;
        let var = letter.chars().take_only()?;
        let cond = match cond {
            "<" => Ordering::Less,
            ">" => Ordering::Greater,
            _ => bail!("Invalid"),
        };
        let value: i64 = value.parse()?;
        debug_assert!(FULL_RANGE.contains(value));
        let range = match cond {
            Ordering::Less => Range::create(FULL_RANGE.start(), value),
            Ordering::Greater => Range::create(value + 1, FULL_RANGE.end()),
            _ => unreachable!(),
        };
        Ok(Test{ text: text.to_string(), var, range, dest: dest.to_string() })
    }
}

impl Debug for Test {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.text)
    }
}

#[derive(Debug)]
struct Workflow {
    name: String,
    tests: Vec<Test>,
    fallback: String,
}

impl Workflow {
    fn apply_part(&self, part: &Part) -> &str {
        for test in &self.tests {
            if test.test_part(part) {
                return &test.dest;
            }
        }
        &self.fallback
    }

    fn apply_range(&self, parts: PartRange) -> (u64, Vec<(&str, PartRange)>) {
        let mut remaining = Some(parts);
        let mut valid = 0;
        let mut tbd = Vec::new();
        for test in &self.tests {
            if let Some(cur) = remaining {
                let (pass, fail) = test.split_range(cur);
                if let Some(pass) = pass {
                    if test.dest == "A" { valid += pass.count(); }
                    else if test.dest != "R" {
                        tbd.push((test.dest.as_str(), pass));
                    }
                }
                remaining = fail;
            } else { break; }
        }
        if let Some(remaining) = remaining {
            if self.fallback == "A" { valid += remaining.count(); }
            else if self.fallback != "R" {
                tbd.push((self.fallback.as_str(), remaining));
            }
        }

        (valid, tbd)
    }
}

impl FromStr for Workflow {
    type Err = Error;

    fn from_str(s: &str) -> Result<Self> {
        let (_, name, tests, fallback) = regex_captures!(r"(.+)\{(.+),([^,]+)\}", s).with_context(|| format!("Invalid: {}", s))?;
        let tests = tests.split(',').map(|t| t.parse()).collect::<Result<Vec<_>>>()?;
        Ok(Workflow{ name: name.to_string(), tests, fallback: fallback.to_string() })
    }
}

struct WorkflowTable {
    workflows: HashMap<String, Workflow>,
}

impl WorkflowTable {
    fn create(items: impl IntoIterator<Item=Workflow>) -> WorkflowTable {
        let workflows = items.into_iter().map(|w| (w.name.to_string(), w)).collect();
        WorkflowTable{ workflows }
    }

    fn validate(&self, part: &Part) -> bool {
        let mut step = "in";
        loop {
            let result = self.workflows[step].apply_part(part);
            if result == "A" { return true; }
            if result == "R" { return false; }
            step = result;
        }
    }

    fn count_valid(&self) -> u64 {
        let mut ranges = VecDeque::from([("in", PartRange::new())]);
        let mut valid = 0;

        while !ranges.is_empty() {
            let (workflow, parts) = ranges.pop_front().expect("Non-empty");
            let (done, tbd) = self.apply_workflow(workflow, parts);
            valid += done;
            ranges.extend(tbd);
        }

        valid
    }

    fn apply_workflow(&self, workflow: &str, parts: PartRange) -> (u64, Vec<(&str, PartRange)>) {
        let workflow = self.workflows.get(workflow)
            .with_context(|| format!("{} not found in {:?}", workflow, self.workflows.keys()))
            .expect("Must be present");
        workflow.apply_range(parts)
    }
}

#[derive(Debug, Copy, Clone)]
struct PartRange {
    x: Range,
    m: Range,
    a: Range,
    s: Range,
}

impl PartRange {
    fn new() -> PartRange { PartRange{ x:FULL_RANGE, m:FULL_RANGE, a:FULL_RANGE, s:FULL_RANGE } }

    fn count(&self) -> u64 {
        self.x.len() * self.m.len() * self.a.len() * self.s.len()
    }

    fn var_range(&self, var: char) -> Range {
        match var {
            'x' => self.x,
            'm' => self.m,
            'a' => self.a,
            's' => self.s,
            _ => panic!(),
        }
    }

    fn constrain(mut self, var: char, range: Range) -> PartRange {
        match var {
            'x' => self.x = range,
            'm' => self.m = range,
            'a' => self.a = range,
            's' => self.s = range,
            _ => panic!(),
        }
        self
    }
}

#[derive(Debug)]
struct Part {
    x: i32,
    m: i32,
    a: i32,
    s: i32,
}

impl Part {
    fn score(&self) -> i32 { self.x + self.m + self.a + self.s }

    fn var(&self, var: char) -> i32 {
        match var {
            'x' => self.x,
            'm' => self.m,
            'a' => self.a,
            's' => self.s,
            _ => panic!(),
        }
    }
}

impl FromStr for Part {
    type Err = Error;

    fn from_str(s: &str) -> Result<Self> {
        let (_, x, m, a, s) = regex_captures!(r"\{x=(\d+),m=(\d+),a=(\d+),s=(\d+)\}", s).with_context(|| format!("Invalid: {}", s))?;
        Ok(Part{ x: x.parse()?, m: m.parse()?, a: a.parse()?, s: s.parse()? })
    }
}

fn parse_input(input: &str) -> Result<(WorkflowTable, Vec<Part>)> {
    let (workflows, parts) = input.split("\n\n").collect_tuple().context("Invalid")?;
    let workflows = WorkflowTable::create(workflows.lines().map(|w| w.parse()).collect::<Result<Vec<_>>>()?);
    let parts = parts.lines().map(|w| w.parse()).collect::<Result<Vec<_>>>()?;
    Ok((workflows, parts))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn check_input() { parse_input(include_str!("input.txt")).unwrap(); }

    #[test]
    fn validate_example() {
        let (workflows, parts) = parse_input(include_str!("example.txt")).unwrap();
        assert_eq!(parts.len(), 5);

        assert!(workflows.validate(&parts[0]));
        assert_eq!(parts[0].score(), 7540);
        assert!(!workflows.validate(&parts[1]));
        assert!(workflows.validate(&parts[2]));
        assert_eq!(parts[2].score(), 4623);
        assert!(!workflows.validate(&parts[3]));
        assert!(workflows.validate(&parts[4]));
        assert_eq!(parts[4].score(), 6951);
    }

    #[test]
    fn count_valid_example() {
        let (workflows, _) = parse_input(include_str!("example.txt")).unwrap();
        assert_eq!(workflows.count_valid(), 167409079868000); // FIXME is this right?
    }
}
