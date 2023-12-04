use std::collections::HashSet;
use std::str::FromStr;
use anyhow::*;
use lazy_regex::regex_captures;

fn main() -> Result<()> {
    let input = parse_input(include_str!("input.txt"))?;
    println!("Total points: {}", input.iter().map(Card::score).sum::<u32>());
    println!("Total cards: {}", count_recursive_wins(&input).iter().sum::<u32>());
    Ok(())
}

#[derive(Debug)]
struct Card {
    #[allow(dead_code)]
    id: u32,
    win: HashSet<u32>,
    numbers: Vec<u32>,
}

impl Card {
    fn winning_nums(&self) -> u32 {
        self.numbers.iter().filter(|n| self.win.contains(n)).count() as u32
    }

    fn score(&self) -> u32 {
        let wins = self.winning_nums();
        if wins == 0 { 0 } else { u32::pow(2, wins - 1) }
    }
}

impl FromStr for Card {
    type Err = Error;

    fn from_str(s: &str) -> Result<Self> {
        let (_, id, win, numbers) = regex_captures!(r"Card\s+(\d+):\s+(.*)\s+\|\s+(.*)", s)
            .with_context(|| format!("No match: {}", s))?;
        let id: u32 = id.parse()?;
        let win = win.split_whitespace().map(|n| n.trim().parse().context("a")).collect::<Result<HashSet<_>>>()?;
        let numbers = numbers.split_whitespace().map(|n| n.parse().context("b")).collect::<Result<Vec<_>>>()?;
        Ok(Card{id, win, numbers})
    }
}

fn count_recursive_wins(cards: &[Card]) -> Vec<u32> {
    let mut counts = vec![1; cards.len()];
    for (i, card) in cards.iter().enumerate() {
        for j in 1..=(card.winning_nums() as usize) {
            counts[i+j] += counts[i];
        }
    }
    counts
}

// Actually play out repeated rounds of won cards - why did I bother implementing this? :D
#[cfg(test)]
fn construct_recursive_wins(cards: &[Card]) -> u32 {
    let mut won: Vec<&Card> = cards.iter().collect();
    let mut total = 0;
    while !won.is_empty() {
        let round = won;
        total += round.len() as u32;
        won = Vec::new();
        for card in &round {
            for i in 0..card.winning_nums() {
                won.push(&cards[(card.id + i) as usize]);
            }
        }
    }
    total
}

fn parse_input(input: &str) -> Result<Vec<Card>> {
    input.lines().map(|l| l.parse()).collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn check_input() { parse_input(include_str!("input.txt")).unwrap(); }

    #[test]
    fn scores() {
        let cards = parse_input(include_str!("example.txt")).unwrap();
        assert_eq!(cards.iter().map(Card::score).collect::<Vec<_>>(), [8, 2, 2, 1, 0, 0]);
    }

    #[test]
    fn recursive_cards() {
        let cards = parse_input(include_str!("example.txt")).unwrap();
        let counts = count_recursive_wins(&cards);
        assert_eq!(counts, [1, 2, 4, 8, 14, 1]);
        assert_eq!(counts.iter().sum::<u32>(), construct_recursive_wins(&cards));
    }
}
