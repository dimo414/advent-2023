use anyhow::*;

fn main() -> Result<()> {
    let input = parse_input(include_str!("input.txt"))?;
    let sums = input.iter()
        .map(|d| extrapolate(d))
        .fold((0, 0), |b, v| (b.0+v.0, b.1+v.1));
    println!("Sum of next values:  {}", sums.1);
    println!("Sum of prior values: {}", sums.0);
    Ok(())
}

fn extrapolate(values: &[i32]) -> (i32, i32) {
    if values.iter().all(|&v| v == 0) {
        return (0, 0);
    }
    let deltas: Vec<_> = values.windows(2).map(|w| w[1] - w[0]).collect();
    let (prior, next) = extrapolate(&deltas);
    (values.first().expect("Non-empty") - prior, values.last().expect("Non-empty") + next)
}

fn parse_input(input: &str) -> Result<Vec<Vec<i32>>> {
    input.lines().map(|l|
        l.split_ascii_whitespace()
            .map(|v| Ok(v.parse::<i32>()?))
            .collect::<Result<Vec<_>>>()
    ).collect::<Result<Vec<_>>>()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn check_input() { parse_input(include_str!("input.txt")).unwrap(); }

    parameterized_test::create!{ example, (data, expected), {
        assert_eq!(extrapolate(&data), expected);
    }}
    example! {
        one: ([0, 3, 6, 9, 12, 15], (-3, 18)),
        two: ([1, 3, 6, 10, 15, 21], (0, 28)),
        three: ([10, 13, 16, 21, 30, 45], (5, 68)),
    }
}
