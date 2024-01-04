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
}