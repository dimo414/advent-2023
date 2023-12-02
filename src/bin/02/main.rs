use std::str::FromStr;
use anyhow::*;
use lazy_regex::regex_captures;

const BAG: Tiles =  Tiles{red: 12, green: 13, blue: 14 };

fn main() -> Result<()> {
    let input = parse_input(include_str!("input.txt"))?;
    let sum_ids: u32 = input.iter().filter(|g| g.is_valid_game_for(&BAG)).map(|g| g.id).sum();
    println!("Valid Games: {}", sum_ids);
    let sum_power: u32 = input.iter().map(|g| g.min_cubes().power()).sum();
    println!("Game Power: {:?}",sum_power);
    Ok(())
}

#[derive(Copy, Clone, Debug)]
struct Tiles {
    red: u32, green: u32, blue: u32,
}

impl Tiles {
    fn power(&self) -> u32 {
        self.red * self.green * self.blue
    }
}

impl FromStr for Tiles {
    type Err = Error;

    fn from_str(s: &str) -> std::result::Result<Self, Self::Err> {
        let mut hand = Tiles {red:0, green:0, blue:0};
        for part in s.split(", ") {
            let cube = part.split(" ").collect::<Vec<_>>();
            ensure!(cube.len() == 2, "{}", part);
            let num: u32 = cube[0].parse()?;
            match cube[1] {
                "red" => { hand.red += num },
                "green" => { hand.green += num },
                "blue" => { hand.blue += num },
                _ => bail!("{}", cube[1]),
            }
        }
        Ok(hand)
    }
}

#[derive(Debug)]
struct Game {
    id: u32,
    hands: Vec<Tiles>,
}

impl Game {
    fn min_cubes(&self) -> Tiles {
        let mut max = Tiles {red:0, green:0, blue:0};
        for hand in &self.hands {
            max.red = std::cmp::max(max.red, hand.red);
            max.green = std::cmp::max(max.green, hand.green);
            max.blue = std::cmp::max(max.blue, hand.blue);
        }
        max
    }

    fn is_valid_game_for(&self, bag: &Tiles) -> bool {
        let min_cubes = self.min_cubes();
        min_cubes.red <= bag.red && min_cubes.green <= bag.green && min_cubes.blue <= bag.blue
    }
}

impl FromStr for Game {
    type Err = Error;

    fn from_str(s: &str) -> Result<Self> {
        match regex_captures!(r"Game ([0-9]+): (.*)", s) {
            Some((_, id, hands)) => {
                Ok(Game{ id: id.parse()?, hands: hands.split("; ").map(|h| h.parse()).collect::<Result<Vec<_>>>()? })
            },
            None => bail!("Invalid Game: {:?}", s),
        }
    }
}

fn parse_input(input: &str) -> Result<Vec<Game>> {
    input.lines().map(|l| l.parse()).collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn check_input() { parse_input(include_str!("input.txt")).unwrap(); }

    #[test]
    fn possible_games() {
        let games = parse_input(include_str!("example.txt")).unwrap();
        let possible = games.iter().filter(|g| g.is_valid_game_for(&BAG)).map(|g| g.id).collect::<Vec<_>>();
        assert_eq!(&possible, &[1, 2, 5]);
    }

    #[test]
    fn game_power() {
        let games = parse_input(include_str!("example.txt")).unwrap();
        let powers = games.iter().map(|g| g.min_cubes().power()).collect::<Vec<_>>();
        assert_eq!(&powers, &[48, 12, 1560, 630, 36]);
    }
}
