// Time:        46     68     98     66
// Distance:   358   1054   1807   1080
static INPUT1: &[Race] = &[
    Race::create(46, 358), Race::create(68, 1054), Race::create(98, 1807), Race::create(66, 1080)];
static INPUT2: Race = Race::create(46689866, 358105418071080);

fn main() {
    println!("Wins Product: {}", INPUT1.iter().map(|r| r.count_wins()).product::<u64>());
    println!("Concated Wins: {}", INPUT2.count_wins());
}

struct Race{
    time: u64,
    target: u64,
}

impl Race {
    const fn create(time: u64, target: u64) -> Race {
        Race{ time, target }
    }

    fn distances(&self) -> impl Iterator<Item=u64> + '_ {
        (0..=self.time).map(|charge| charge * (self.time - charge))
    }

    fn count_wins(&self) -> u64 {
        self.distances().filter(|&d| d > self.target).count() as u64
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    // Time:      7  15   30
    // Distance:  9  40  200
    static EXAMPLE1: &[Race] = &[
        Race::create(7, 9), Race::create(15, 40), Race::create(30, 200)];
    static EXAMPLE2: Race = Race::create(71530, 940200);

    #[test]
    fn check_distances() {
        let example1_1 = &EXAMPLE1[0];
        assert_eq!(example1_1.distances().collect::<Vec<_>>(), [0, 6, 10, 12, 12, 10, 6, 0]);
    }

    #[test]
    fn count_wins() {
        let wins: Vec<_> = EXAMPLE1.iter().map(|r| r.count_wins()).collect();
        assert_eq!(wins, [4, 8, 9]);
    }

    #[test]
    fn count_wins_concated() {
        assert_eq!(EXAMPLE2.count_wins(), 71503);
    }
}
