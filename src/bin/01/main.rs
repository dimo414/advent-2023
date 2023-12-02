use std::collections::HashMap;
use anyhow::*;
use once_cell::sync::Lazy;
use regex::Regex;

const DIGITS: &'static [&'static str] = &["0", "1", "2", "3", "4", "5", "6", "7", "8", "9"];
const WORDS: &'static [&'static str] = &["zero", "one", "two", "three", "four", "five", "six", "seven", "eight", "nine"];

const LOOKUP: Lazy<HashMap<&'static str, u32>> = Lazy::new(||
    DIGITS.iter().enumerate().chain(WORDS.iter().enumerate()).map(|(i,&w)| (w, i as u32)).collect());

static DIGITS_RE: Lazy<Regex> = Lazy::new(|| Regex::new(&DIGITS.join("|")).unwrap());
static WORDS_RE: Lazy<Regex> = Lazy::new(||
    Regex::new(&format!("{}|{}", DIGITS.join("|"), WORDS.join("|"))).unwrap());

fn main() -> Result<()> {
    let digit_sum: u32 = include_str!("input.txt").lines().map(|l| extract_number(l, &DIGITS_RE).unwrap()).sum();
    println!("Initial Calibration Sum: {}", digit_sum);
    let word_sum: u32 = include_str!("input.txt").lines().map(|l| extract_number(l, &WORDS_RE).unwrap()).sum();
    println!("Updated Calibration Sum: {}", word_sum);

    Ok(())
}

fn tail_find<'a>(line: &'a str, re: &Regex) -> Option<&'a str> {
    for i in (0..line.len()).rev() {
        //if let Some(m) = re.find_at(line, i) {
        if let Some(m) = re.find(&line[i..std::cmp::min(line.len(), i+5)]) {
            return Some(m.as_str());
        }
    }
    None
}

fn extract_number(line: &str, re: &Regex) -> Result<u32> {
    let head = re.find(line).context("No digit found looking forwards")?.as_str();
    let tail = tail_find(line, re).context("No digit found looking backwards")?;
    Ok(LOOKUP[head] * 10 + LOOKUP[tail])
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::path::PathBuf;
    use std::process::Command;

    fn extract_numbers(s: &str, re: &Regex) -> Result<Vec<u32>> {
        s.lines().map(|l| extract_number(l, re)).collect()
    }

    #[test]
    fn extract_digits() {
        assert_eq!(extract_numbers(include_str!("example1.txt"), &DIGITS_RE).unwrap(), &[12, 38, 15, 77]);
    }

    #[test]
    fn extract_words() {
        assert_eq!(extract_numbers(include_str!("example1.txt"), &DIGITS_RE).unwrap(), &[12, 38, 15, 77]);
        assert_eq!(extract_numbers(include_str!("example2.txt"), &WORDS_RE).unwrap(), &[29, 83, 13, 24, 42, 14, 76]);
    }

    #[test]
    fn bash_solution() {
        // TODO pull this into a util
        let dir = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("src/bin/01");
        let res = Command::new("bash")
            .arg(dir.join("01.sh"))
            .arg(dir.join("input.txt"))
            .output().unwrap();

        assert!(res.status.success());
        assert!(res.stderr.is_empty());
        assert_eq!(String::from_utf8(res.stdout).unwrap(), "Part 1:\t53921\nPart 2:\t54676\n");
    }
}
