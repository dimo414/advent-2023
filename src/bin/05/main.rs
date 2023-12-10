use std::ops::Range;
use std::str::FromStr;
use anyhow::*;
use itertools::Itertools;
use range_collections::{RangeSet, RangeSet2};
use range_collections::range_set::RangeSetRange;

fn main() -> Result<()> {
    let (seeds, almanac) = parse_input(include_str!("input.txt"))?;
    println!("Min Location: {}", min_location(&seeds,&almanac));
    println!("Min Range Location: {}", min_location_ranges(&seeds, &almanac));

    Ok(())
}

#[derive(Debug)]
struct Mapping {
    source: Range<i64>,
    delta: i64,
}

impl FromStr for Mapping {
    type Err = Error;

    fn from_str(s: &str) -> Result<Self> {
        let (dest, source, len) = s.split(' ').collect_tuple().context("Invalid format")?;
        let dest = dest.parse::<i64>()?;
        let source = source.parse()?;
        let delta = dest - source;
        let len = len.parse::<i64>()?;
        Ok(Mapping{ source: source..(source+len), delta })
    }
}

#[derive(Debug)]
struct Almanac {
    mappings: Vec<Vec<Mapping>>,
}

impl FromStr for Almanac {
    type Err = Error;

    fn from_str(s: &str) -> Result<Self> {
        let mut mappings = Vec::new();
        for line in s.lines() {
            if line.is_empty() {
                // skip
            } else if line.contains(" map:") {
                mappings.push(Vec::new());
            } else {
                assert!(!mappings.is_empty());
                let mapping: Mapping = line.parse()?;
                let i = mappings.len()-1;
                mappings[i].push(mapping);
            }
        }
        Ok(Almanac{ mappings })
    }
}

fn min_location(seeds: &[i64], almanac: &Almanac) -> i64 {
    let mut min = i64::MAX;
    for seed in seeds {
        let mut id = *seed;
        for mapping in &almanac.mappings {
            let next_id = transform(mapping, id);
            id = next_id;
        }
        min = std::cmp::min(min, id);
    }
    min
}

fn min_location_ranges(seeds: &[i64], almanac: &Almanac) -> i64 {
    let mut min = i64::MAX;
    for chunk in seeds.chunks(2) {
        let (start, len) = chunk.iter().collect_tuple().expect("2-chunks");
        let mut ranges: RangeSet2<i64> = RangeSet::from(*start..start+len);
        for mapping in &almanac.mappings {
            let mut next_ranges = RangeSet2::empty();
            for range in ranges.iter() {
                let range = to_range(&range.cloned());
                next_ranges |= transform_range(mapping, &range);
            }
            ranges = next_ranges;
        }
        min = std::cmp::min(min, to_range(&ranges.iter().next().expect("Not-empty").cloned()).start);
    }
    min
}

fn transform(mappings: &[Mapping], id: i64) -> i64 {
    for mapping in mappings {
        if mapping.source.contains(&id) {
            return id + mapping.delta;
        }
    }
    id // unchanged
}

fn transform_range(mappings: &[Mapping], ids: &Range<i64>) -> RangeSet2<i64> {
    let mut remaining = RangeSet::from(ids.clone());
    let mut moved = RangeSet::empty();
    for mapping in mappings {
        if ids.start < mapping.source.end && ids.end > mapping.source.start {
            let r = std::cmp::max(ids.start, mapping.source.start)..std::cmp::min(ids.end, mapping.source.end);
            moved |= RangeSet::from((r.start+mapping.delta)..(r.end+mapping.delta));
            remaining -= RangeSet::from(r);
        }
    }
    let ret = &moved | &remaining;
    debug_assert_eq!(ids.end-ids.start, rangeset_len(&ret),"In:{} Out:{}+{}={}", ids.end-ids.start, rangeset_len(&moved), rangeset_len(&remaining), rangeset_len(&ret));
    ret
}

fn rangeset_len(set: &RangeSet2<i64>) -> i64 {
    set.iter().map(|r| match r {
        RangeSetRange::Range(r) => r.end - r.start,
        RangeSetRange::RangeFrom(_) => panic!("Should not be inserting unbounded ranges"),
    })
    .sum()
}

fn to_range(range: &RangeSetRange<i64>) -> Range<i64> {
    match range {
        RangeSetRange::Range(ref range) => range.clone(),
        RangeSetRange::RangeFrom(_) => panic!("Should not be inserting unbounded ranges"),
    }
}

fn parse_input(input: &str) -> Result<(Vec<i64>, Almanac)> {
    let (seeds, almanac) = input.splitn(2, "\n\n").collect_tuple().context("Invalid")?;
    let seeds = seeds.strip_prefix("seeds: ").context("No prefix")?
        .split(' ').map(|n| n.parse::<i64>().context("")).collect::<Result<Vec<_>>>()?;
    let almanac: Almanac = almanac.parse()?;
    Ok((seeds, almanac))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn check_input() { parse_input(include_str!("input.txt")).unwrap(); }

    #[test]
    fn value() {
        let (seeds, almanac) = parse_input(include_str!("example.txt")).unwrap();
        assert_eq!(min_location_ranges(&seeds, &almanac), 46);
    }

    #[test]
    fn range() {
        let (seeds, almanac) = parse_input(include_str!("example.txt")).unwrap();
        assert_eq!(min_location_ranges(&seeds, &almanac), 46);
    }
}
