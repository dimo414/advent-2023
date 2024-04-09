use std::fmt::{Debug, Formatter};
use std::str::FromStr;
use anyhow::*;
use itertools::Itertools;

fn main() -> Result<()> {
    let input = parse_input(include_str!("example.txt"))?;
    println!("HASH Sum: {}", input.iter().map(|s| holiday_ascii_string_helper(s) as u32).sum::<u32>());

    let input = to_ops(&input)?;
    println!("Focusing Power: {}", execute(&input));
    Ok(())
}

fn holiday_ascii_string_helper(s: &str) -> u8 {
    s.bytes().fold(0, |cv, c| cv.wrapping_add(c).wrapping_mul(17))
}

#[derive(Debug)]
enum Op {
    Rm(String),
    Add(String, u8),
}

impl FromStr for Op {
    type Err = Error;

    fn from_str(s: &str) -> Result<Self> {
        if let Some(s) = s.strip_suffix('-') {
           Ok(Op::Rm(s.to_string()))
        } else {
            let (s, f) = s.split('=').collect_tuple().context("Invalid")?;
            ensure!(!s.is_empty());
            Ok(Op::Add(s.to_string(), f.parse()?))
        }
    }
}

struct Hashmap<'a> {
    data: [Vec<(&'a str, u8)>; 256],
}

impl<'a> Hashmap<'a> {
    fn create() -> Hashmap<'a> {
        Hashmap{ data: std::array::from_fn(|_| Vec::new()) }
    }

    fn add(&mut self, key: &'a str, value: u8) {
        let idx = holiday_ascii_string_helper(key) as usize;
        if let Some(pos) = self.data[idx].iter().position(|(old_key,_)| old_key == &key) {
            self.data[idx][pos].1 = value;
        } else {
            self.data[idx].push((key, value));
        }
    }

    fn rm(&mut self, key: &str) {
        let idx = holiday_ascii_string_helper(key) as usize;
        if let Some(pos) = self.data[idx].iter().position(|(old_key,_)| old_key == &key) {
            self.data[idx].remove(pos);
        }
    }

    fn power(&self) -> u32 {
        self.data.iter().enumerate().flat_map(|(idx, bucket)|
            bucket.iter().enumerate().map(move |(pos, (_, f))|
                (idx as u32 + 1) * (pos as u32 + 1) * (*f as u32)
            )).sum()
    }
}

impl<'a> Debug for Hashmap<'a> {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        for (idx, bucket) in self.data.iter().enumerate() {
            if !bucket.is_empty() {
                writeln!(f, "{}: {}", idx, bucket.iter().map(|(k, v)| format!("[{} {}]", k, v)).join(" "))?;
            }
        }
        Result::Ok(())
    }
}

fn execute(ops: &[Op]) -> u32 {
    let mut hashmap = Hashmap::create();
    for op in ops {
        match op {
            Op::Rm(s) => hashmap.rm(s),
            Op::Add(s, f) => hashmap.add(s, *f),
        }
        //println!("{:?}", hashmap);
    }
    hashmap.power()
}

fn parse_input(input: &str) -> Result<Vec<String>> {
    Ok(input.split(',').map(|s| s.into()).collect())
}

fn to_ops(input: &[String]) -> Result<Vec<Op>> {
    input.iter().map(|o| o.parse()).collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn check_input() {
        to_ops(&parse_input(include_str!("input.txt")).unwrap()).unwrap();
    }

    parameterized_test::create!{ hash, (s, h), { assert_eq!(holiday_ascii_string_helper(s), h); } }
    hash! {
        a: ("HASH", 52),
    }

    #[test]
    fn hashes() {
        let example = parse_input(include_str!("example.txt")).unwrap();
        assert_eq!(
            example.iter().map(|s| holiday_ascii_string_helper(s)).collect::<Vec<_>>(),
            [30, 253, 97, 47, 14, 180, 9, 197, 48, 214, 231]);
    }

    #[test]
    fn ops() {
        let example = to_ops(&parse_input(include_str!("example.txt")).unwrap()).unwrap();
        assert_eq!(execute(&example), 145);
    }
}
