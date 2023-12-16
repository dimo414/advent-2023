use std::collections::HashMap;
use std::str::FromStr;
use anyhow::*;
use itertools::Itertools;
use regex::Regex;
use advent_2023::elapsed;

fn main() -> Result<()> {
    let input = parse_input(include_str!("input.txt"))?;
    if cfg!(feature="timing") {
        elapsed!("Construct valid", input.iter().map(|r| r.construct_valid_rows().len()).sum::<usize>());
        elapsed!("Count valid", input.iter().map(|r| r.count_valid_rows()).sum::<u64>());
    }

    println!("Possible arrangements: {}", elapsed!("Count cached", count_valid_rows_cached(&input).iter().sum::<u64>()));

    let input = parse_unfolded_input(include_str!("input.txt"))?;
    if cfg!(feature="timing") {
        let partial = &input[0..3];
        elapsed!("Construct valid unfolded (0..3)", partial.iter().map(|r| r.construct_valid_rows().len()).sum::<usize>());
        elapsed!("Count valid unfolded (0..3)", partial.iter().map(|r| r.count_valid_rows()).sum::<u64>());
        elapsed!("Count cached unfolded (0..3)", count_valid_rows_cached(partial).iter().sum::<u64>());
    }

    println!("Possible unfolded arrangements: {}", elapsed!("Count cached unfolded", count_valid_rows_cached(&input).iter().sum::<u64>()));

    Ok(())
}

fn create_regex(damaged: &[usize]) -> Regex {
    let interior = damaged.iter().map(|n| format!("[#?]{{{}}}", n)).join("[.?]+");
    Regex::new(&format!("^[.?]*{}[.?]*$", interior)).expect("Invalid")
}

fn count_valid_rows_cached(records: &[Record]) -> Vec<u64> {
    let mut ret = Vec::new();
    let mut cache = HashMap::new();

    for record in records {
        ret.push(valid_rows(&record.row, &record.damaged, &mut cache));
    }
    ret
}

// Verify row starts with `damage` non-working slots, followed by a working slot or the end of the
// string. Returns length that has been checked.
fn valid_damage_prefix(row: &str, damage: usize) -> Option<usize> {
    if row.len() < damage { return None; }
    // there cannot be any . once we've started a series of #, and all ?s must be #
    if row[..damage].contains('.') { return None; }
    if row.len() > damage {
        // and the following slot, if any, cannot be a #
        if &row[damage..damage+1] == "#" { return None; }
        return Some(damage+1);
    }
    debug_assert_eq!(row.len(), damage);
    Some(damage)
}

fn valid_rows<'a>(row: &'a str, damaged: &'a [usize], cache: &mut HashMap<(&'a str, &'a [usize]), u64>) -> u64 {
    //println!("\tConsidering: {:?} {:?}", row, damaged);
    if let Some(count) = cache.get(&(row, damaged)) {
        return *count;
    }
    if damaged.is_empty() {
        // can't be any more damaged locations in the string
        let count = if row.contains('#') { 0 } else { 1 };
        cache.insert((row, damaged), count);
        return count;
    }
    // Ran out of row but still expected damaged runs
    if row.is_empty() { debug_assert!(!damaged.is_empty()); return 0; }

    let head = row.chars().next().expect("Empty string");
    let mut dot_count = 0;
    let mut hash_count = 0;
    if head == '.' || head == '?' {
        dot_count = valid_rows(&row[1..], damaged, cache);
    }
    // This is always the first # in a sequence; we never consider part-way through a run of damaged locations
    if head == '#' || head == '?' {
        if let Some(shift) = valid_damage_prefix(row, damaged[0]) {
            hash_count = valid_rows(&row[shift..], &damaged[1..], cache)
        }
    }
    cache.insert((row, damaged), dot_count + hash_count);
    dot_count + hash_count
}

#[derive(Debug)]
struct Record {
    row: String,
    damaged: Vec<usize>,
}

impl Record {
    fn create(row: String, damaged: Vec<usize>) -> Result<Record> {
        if cfg!(debug) {
            let validator = create_regex(&damaged);
            ensure!(validator.is_match(&row), "Row '{}' is not valid as {:?} per\n\t{}", row, damaged, validator);
        }
        Ok(Record{row, damaged})
    }

    fn construct_valid_rows(&self) -> Vec<String> {
        let validator = create_regex(&self.damaged);
        let mut candidates = vec![self.row.clone()];
        while candidates[0].contains('?') {
            let prior = candidates;
            candidates = Vec::new();
            for candidate in prior {
                let add_working = candidate.replacen('?', ".", 1);
                if validator.is_match(&add_working) {
                    candidates.push(add_working);
                }
                let add_broken = candidate.replacen('?', "#", 1);
                if validator.is_match(&add_broken) {
                    candidates.push(add_broken);
                }
            }
        }
        candidates
    }

    fn count_valid_rows(&self) -> u64 {
        fn valid_rows(candidate: &str, validator: &Regex) -> u64 {
            if !candidate.contains('?') { return 1; }
            let mut ret = 0;
            let add_working = candidate.replacen('?', ".", 1);
            if validator.is_match(&add_working) {
                ret += valid_rows(&add_working, validator);
            }
            let add_broken = candidate.replacen('?', "#", 1);
            if validator.is_match(&add_broken) {
                ret += valid_rows(&add_broken, validator);
            }
            ret
        }
        valid_rows(&self.row, &create_regex(&self.damaged))
    }
}

impl FromStr for Record {
    type Err = Error;

    fn from_str(s: &str) -> Result<Self> {
        let (row, damaged) = s.split(' ').collect_tuple().context("Invalid")?;
        let damaged = damaged.split(',').map(|n| n.parse().context("Invalid")).collect::<Result<Vec<_>>>()?;
        Record::create(row.into(), damaged)
    }
}

fn unfold_line(line: &str) -> Result<String> {
    let (left, right) = line.split(' ').collect_tuple().context("Invalid")?;
    let left = std::iter::repeat(left).take(5).join("?");
    let right = std::iter::repeat(right).take(5).join(",");
    Ok(format!("{} {}", left, right))
}

fn parse_input(input: &str) -> Result<Vec<Record>> {
    input.lines().map(|l| l.parse()).collect()
}

fn parse_unfolded_input(input: &str) -> Result<Vec<Record>> {
    input.lines().map(|l| unfold_line(l).and_then(|u| u.parse())).collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn check_input() { parse_input(include_str!("input.txt")).unwrap(); }

    parameterized_test::create!{ examples, (input, valid, valid_unfolded), {
        let mut cache = HashMap::new();
        let record: Record = input.parse().unwrap();
        let candidates = record.construct_valid_rows();
        assert_eq!(candidates.len() as u64, valid, "{:?}", candidates);
        assert_eq!(record.count_valid_rows(), valid);
        assert_eq!(valid_rows(&record.row, &record.damaged, &mut cache), valid);

        let record: Record = unfold_line(input).unwrap().parse().unwrap();
        assert_eq!(valid_rows(&record.row, &record.damaged, &mut cache), valid_unfolded);
    } }
    examples! {
        a_a: ("#.#.### 1,1,3", 1, 1),
        a_b: (".#...#....###. 1,1,3", 1, 1),
        a_c: (".#.###.#.###### 1,3,1,6", 1, 1),
        a_d: ("####.#...#... 4,1,1", 1, 1),
        a_e: ("#....######..#####. 1,6,5", 1, 1),
        a_f: (".###.##....# 3,2,1", 1, 1),
        b_a: ("???.### 1,1,3", 1, 1),
        b_b: (".??..??...?##. 1,1,3", 4, 16384),
        b_c: ("?#?#?#?#?#?#?#? 1,3,1,6", 1, 1),
        b_d: ("????.#...#... 4,1,1", 1, 16),
        b_e: ("????.######..#####. 1,6,5", 4, 2500),
        b_f: ("?###???????? 3,2,1", 10, 506250),
    }

    parameterized_test::create!{ unfold, (input, expected), {
        let unfolded = unfold_line(input).unwrap();
        assert_eq!(unfolded, expected);
        unfolded.parse::<Record>().unwrap();
    }}
    unfold! {
        a: (".# 1", ".#?.#?.#?.#?.# 1,1,1,1,1"),
        b: ("???.### 1,1,3", "???.###????.###????.###????.###????.### 1,1,3,1,1,3,1,1,3,1,1,3,1,1,3"),
    }
}
