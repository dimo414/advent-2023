use std::cell::Cell;
use std::cmp::Ordering;
use std::collections::HashMap;
use std::str::FromStr;
use anyhow::*;
use itertools::Itertools;

fn main() -> Result<()> {
    let mut input = parse_input(include_str!("input.txt"))?;
    input.sort();
    println!("Winnings: {}", input.iter().enumerate().map(|(i, (_, b))| b*(i as u64+1)).sum::<u64>());

    let mut input = parse_input(&jacks_wild(include_str!("input.txt")))?;
    input.sort();
    println!("Wildcard Winnings: {}", input.iter().enumerate().map(|(i, (_, b))| b*(i as u64+1)).sum::<u64>());

    Ok(())
}

fn jacks_wild(hands: &str) -> String {
    hands.replace('J', "W")
}

fn card_rank(c: char) -> usize {
    match c {
        'A' => 13,
        'K' => 12,
        'Q' => 11,
        'J' => 10,
        'T' => 9,
        '9' => 8,
        '8' => 7,
        '7' => 6,
        '6' => 5,
        '5' => 4,
        '4' => 3,
        '3' => 2,
        '2' => 1,
        'W' => 0,
        _ => panic!(),
    }
}

fn cmp_cards(a: char, b: char) -> Ordering {
    card_rank(a).cmp(&card_rank(b))
}

#[derive(Ord, PartialOrd, Eq, PartialEq, Debug, Copy, Clone)]
enum Type {
    High,
    OnePair,
    TwoPair,
    Three,
    Full,
    Four,
    Five,
}

#[derive(Debug)]
struct Hand {
    str: String,
    _type: Cell<Option<Type>>,
}

impl Hand {
    fn create(str: String) -> Hand {
        debug_assert!(!str.contains('J') || !str.contains('W'), "Invalid hand contains Jacks and Wilds: {}", str);
        Hand{str, _type: Cell::new(None)}
    }

    fn hand_type(&self) -> Type {
        if let Some(t) = self._type.get() { return t; }
        let mut card_counts = HashMap::new();
        for c in self.str.chars() {
            *card_counts.entry(c).or_insert(0) += 1;
        }
        debug_assert_eq!(card_counts.values().sum::<usize>(), 5);
        let mut counts = card_counts.iter()
            .filter_map(|(&c,&n)| if c == 'W' { None } else { Some(n) })
            .sorted().collect::<Vec<usize>>();
        if counts.is_empty() {
            counts.push(0);
        }
        *counts.last_mut().expect("non-empty") += card_counts.get(&'W').unwrap_or(&0);
        debug_assert_eq!(counts.iter().sum::<usize>(), 5);
        let t = match &counts[..] {
            [5] => Type::Five,
            [1, 4] => Type::Four,
            [2, 3] => Type::Full,
            [1, 1, 3] => Type::Three,
            [1, 2, 2] => Type::TwoPair,
            [1, 1, 1, 2] => Type::OnePair,
            [1, 1, 1, 1, 1] => Type::High,
            _ => panic!(),
        };
        self._type.replace(Some(t));
        t
    }
}

impl Ord for Hand {
    fn cmp(&self, other: &Self) -> Ordering {
        let hand_ord = self.hand_type().cmp(&other.hand_type());
        if !hand_ord.is_eq() { return hand_ord; }
        if let Some(ord) = self.str.chars().zip(other.str.chars())
            .map(|(a, b)| cmp_cards(a, b)).find(|o| !o.is_eq()) {
            return ord;
        }
        Ordering::Equal
    }
}

impl PartialOrd for Hand {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

impl PartialEq for Hand {
    fn eq(&self, other: &Self) -> bool {
        self.str == other.str
    }
}

impl Eq for Hand { }

impl FromStr for Hand {
    type Err = Error;

    fn from_str(str: &str) -> Result<Self> {
        Ok(Hand::create(str.into()))
    }
}

fn parse_input(input: &str) -> Result<Vec<(Hand, u64)>> {
    input.lines().map(|line| {
        let (l, r) = line.split(' ').collect_tuple().context("Invalid")?;
        Ok((l.parse()?, r.parse()?))
    }).collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn check_input() { parse_input(include_str!("input.txt")).unwrap(); }

    parameterized_test::create!{ types, (hand, expected), {
        let hand = Hand::create(hand.to_string());
        assert_eq!(hand.hand_type(), expected);
    } }
    types! {
        five: ("AAAAA", Type::Five),
        four: ("AA8AA", Type::Four),
        full: ("23332", Type::Full),
        three: ("TTT98", Type::Three),
        two_p: ("23432", Type::TwoPair),
        one_p: ("A23A4", Type::OnePair),
        high: ("23456", Type::High),
        wild: (jacks_wild("QJJQ2"), Type::Four),
    }

    parameterized_test::create!{ strength, (a, b, ord), {
        let a = Hand::create(a.into());
        let b = Hand::create(b.into());
        assert_eq!(a.cmp(&b), ord);
    }}
    strength!{
        a: ("33332", "2AAAA", Ordering::Greater),
        b: ("77888", "77788", Ordering::Greater),
        wild: (jacks_wild("JKKK2"), "QQQQ2", Ordering::Less),
        diff_types: ("77888", "33332", Ordering::Less),
    }

    #[test]
    fn part1() {
        let input = parse_input(include_str!("example.txt")).unwrap();
        let ordered: Vec<_> = input.iter().sorted()
            .map(|(h, b)| (h.str.as_str(), *b)).collect();
        assert_eq!(ordered, [("32T3K", 765), ("KTJJT", 220), ("KK677", 28), ("T55J5", 684), ("QQQJA", 483)]);
    }

    #[test]
    fn part2() {
        let input = parse_input(&jacks_wild(include_str!("example.txt"))).unwrap();
        let  ordered: Vec<_> = input.iter().sorted()
            .map(|(h, b)| (h.str.as_str(), *b)).collect();
        assert_eq!(ordered, [("32T3K", 765), ("KK677", 28), ("T55W5", 684), ("QQQWA", 483), ("KTWWT", 220)]);
    }
}
