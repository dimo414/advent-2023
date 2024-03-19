use std::collections::HashMap;
use std::fmt::Debug;
use std::hash::Hash;
use std::num::NonZeroUsize;
use anyhow::{anyhow, Result};
use itertools::Itertools;

pub trait MoreItertools : Itertools {
    // Consumes the only element in the iterator, returning an error if iterator does not contain
    // exactly one element. See also Itertools::exactly_one() which this wraps.
    fn drain_only(self) -> Result<Self::Item>
        where Self: Sized, <Self as Iterator>::Item: Debug,
    {
        self.exactly_one().map_err(|e| anyhow!("Unexpected contents: {:?}", e.collect::<Vec<_>>()))
    }
}
impl<T: ?Sized> MoreItertools for T where T: Iterator { }

pub trait MoreIntoIterator : IntoIterator {
    // Consumes a collection and returns its only element. See also Itertools::exactly_one().
    fn take_only(self) -> Result<Self::Item>
        where Self: Sized, <Self as IntoIterator>::Item: Debug
    {
        self.into_iter().drain_only()
    }
}
impl<T: ?Sized> MoreIntoIterator for T where T: IntoIterator { }

pub struct DisjointSet<E> {
    nodes: Vec<E>,
    reverse: HashMap<E, usize>,
    parents: Vec<(usize, Option<NonZeroUsize>)>,
}

impl<E: Clone+Hash+Eq> DisjointSet<E> {
    pub fn create(elements: impl IntoIterator<Item=E>) -> DisjointSet<E> {
        let nodes: Vec<_> = elements.into_iter().collect();
        // O(n) clones :/ - it's better than a clone-per-edge, but it's still not great
        let reverse = nodes.iter().enumerate().map(|(i, n)| (n.clone(), i)).collect();
        let parents: Vec<_> = (0..nodes.len()).map(|n| (n, NonZeroUsize::new(1))).collect();
        DisjointSet{ nodes, reverse, parents }
    }

    fn find_idx(&mut self, idx: usize) -> (usize, NonZeroUsize) {
        let parent = self.parents[idx];
        if parent.0 == idx { return (parent.0, parent.1.expect("Root size is known")); }
        let root = self.find_idx(parent.0);
        self.parents[idx] = (root.0, None);
        root
    }

    pub fn find(&mut self, e: &E) -> &E {
        let e = self.reverse[e];
        let (root, _) = self.find_idx(e);
        &self.nodes[root]
    }

    pub fn set_size(&mut self, e: &E) -> usize {
        let e = self.reverse[e];
        let (_, size) = self.find_idx(e);
        size.get()
    }

    fn union_idx(&mut self, a: usize, b: usize) -> bool {
        let mut a = self.find_idx(a);
        let mut b = self.find_idx(b);
        if a == b { return false; } // already same set
        if a.1 < b.1 {
            // ensure b is smaller than a
            std::mem::swap( &mut a, &mut b);
        }
        // make a the new root for b
        self.parents[b.0] = (a.0, None);
        self.parents[a.0] = (a.0, Some(a.1.checked_add(b.1.get()).expect("Too big")));
        true
    }

    pub fn union(&mut self, a: &E, b: &E) -> bool {
        self.union_idx(self.reverse[a], self.reverse[b])
    }

    pub fn roots(&self) -> Vec<&E> {
        self.parents.iter().enumerate()
            .filter(|(k, v)| *k == v.0)
            .map(|(k, _)| &self.nodes[k])
            .collect()
    }
}

#[derive(Debug, Eq, PartialEq)]
pub enum Difference {
    None,
    One(Range),
    Two(Range, Range),
}

// A closed-open interval from start to end.
#[derive(Debug, Copy, Clone, Eq, PartialEq, Ord, PartialOrd)]
pub struct Range {
    start: i64,
    end: i64,
}

impl Range {
    pub const fn create(start: i64, end: i64) -> Range { assert!(start <= end); Range{ start, end } }

    pub fn start(&self) -> i64 { self.start }
    pub fn end(&self) -> i64 { self.end }

    pub fn len(&self) -> u64 {
        (self.end - self.start) as u64
    }

    pub fn contains(&self, value: i64) -> bool {
        value >= self.start && value < self.end
    }

    pub fn intersect(&self, other: Range) -> Option<Range> {
        if self.start > other.start { return other.intersect(*self); }
        debug_assert!(self.start <= other.start);
        if self.end <= other.start { return None; }
        Some(Range::create(other.start, std::cmp::min(self.end, other.end)))
    }

    pub fn difference(&self, other: Range) -> Difference {
        let left = Range::create(i64::MIN, other.start);
        let right = Range::create(other.end, i64::MAX);
        match (self.intersect(left), self.intersect(right)) {
            (Some(left), Some(right)) => Difference::Two(left, right),
            (Some(r), None) | (None, Some(r)) => Difference::One(r),
            (None, None) => Difference::None,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn drain_only_test() {
        assert_eq!((1..1).drain_only().unwrap_err().to_string(), "Unexpected contents: []");
        assert_eq!((1..=1).drain_only().unwrap(), 1);
        assert_eq!((1..=3).drain_only().unwrap_err().to_string(), "Unexpected contents: [1, 2, 3]");
    }

    #[test]
    fn take_only_test() {
        let empty: &[i32] = &[];
        assert_eq!(empty.take_only().unwrap_err().to_string(), "Unexpected contents: []");
        assert_eq!(&[1].take_only().unwrap(), &1);
        assert_eq!(&[1, 2, 3].take_only().unwrap_err().to_string(), "Unexpected contents: [1, 2, 3]");
    }

    #[test]
    fn disjoint() {
        let mut sets = DisjointSet::create([1, 2, 3, 4, 5, 6, 7 ,8]);
        assert_eq!(sets.roots().len(), 8);
        assert_eq!(sets.set_size(&1), 1);
        assert_eq!(sets.union(&1, &8), true);
        assert_eq!(sets.union(&1, &8), false);
        assert_eq!(sets.set_size(&1), 2);
        assert_eq!(sets.set_size(&8), 2);
        assert_eq!(sets.set_size(&2), 1);
        sets.union(&3, &4);
        sets.union(&2, &6);
        sets.union(&6, &5);
        sets.union(&5, &1);
        assert_eq!(sets.roots().len(), 3);
        assert_eq!(sets.set_size(&1), 5);
        assert_eq!(sets.set_size(&4), 2);
        assert_eq!(sets.set_size(&7), 1);
    }

    mod ranges {
        use super::*;

        #[test]
        fn len() {
            assert_eq!(Range::create(5, 10).len(), 5);
            assert_eq!(Range::create(0, 1).len(), 1);
        }

        #[test]
        fn contains() {
            let r10_20 = Range::create(10, 20);
            assert!(r10_20.contains(10));
            assert!(r10_20.contains(15));
            assert!(r10_20.contains(19));

            assert!(!r10_20.contains(20));
            assert!(!r10_20.contains(5));
            assert!(!r10_20.contains(25));
        }

        #[test]
        fn intersects() {
            let r10_20 = Range::create(10, 20);
            let r20_25 = Range::create(20, 25);
            let r15_25 = Range::create(15, 25);
            let r12_16 = Range::create(12, 16);
            let r0_1 = Range::create(0, 1);

            assert_eq!(r10_20.intersect(r15_25), Some(Range::create(15, 20)));
            assert_eq!(r10_20.intersect(r20_25), None);
            assert_eq!(r20_25.intersect(r10_20), None);
            assert_eq!(r10_20.intersect(r12_16), Some(r12_16));
            assert_eq!(r10_20.intersect(r0_1), None);
            assert_eq!(r15_25.intersect(r12_16), Some(Range::create(15, 16)));
            assert_eq!(r0_1.intersect(r12_16), None);
        }

        #[test]
        fn difference() {
            let r10_20 = Range::create(10, 20);
            let r15_20 = Range::create(15, 20);
            let r15_25 = Range::create(15, 25);
            let r12_16 = Range::create(12, 16);
            let r0_1 = Range::create(0, 1);

            assert_eq!(r10_20.difference(r15_25), Difference::One(Range::create(10, 15)));
            assert_eq!(r15_25.difference(r10_20), Difference::One(Range::create(20, 25)));

            assert_eq!(r10_20.difference(r15_20), Difference::One(Range::create(10, 15)));
            assert_eq!(r15_20.difference(r10_20), Difference::None);

            assert_eq!(r10_20.difference(r12_16), Difference::Two(Range::create(10, 12), Range::create(16, 20)));
            assert_eq!(r12_16.difference(r10_20), Difference::None);

            assert_eq!(r10_20.difference(r0_1), Difference::One(r10_20));
            assert_eq!(r0_1.difference(r10_20), Difference::One(r0_1));

            assert_eq!(r15_20.difference(r15_25), Difference::None);
            assert_eq!(r15_25.difference(r15_20), Difference::One(Range::create(20, 25)));

            assert_eq!(r15_25.difference(r12_16), Difference::One(Range::create(16, 25)));
            assert_eq!(r12_16.difference(r15_25), Difference::One(Range::create(12, 15)));



        // Unexpected split, Range { start: 1, end: 4001 }-Range { start: 2382, end: 4001 }
            // (Range { start: 1, end: 2382 },Range { start: 4001, end: 4001 }
        }
    }
}
