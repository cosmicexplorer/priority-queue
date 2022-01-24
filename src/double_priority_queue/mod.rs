/*
 *  Copyright 2017, 2022 Gianmarco Garrisi
 *
 *
 *  This program is free software: you can redistribute it and/or modify
 *  it under the terms of the GNU Lesser General Public License as published by
 *  the Free Software Foundation, either version 3 of the License, or
 *  (at your option) any later version, or (at your opinion) under the terms
 *  of the Mozilla Public License version 2.0.
 *
 *  This program is distributed in the hope that it will be useful,
 *  but WITHOUT ANY WARRANTY; without even the implied warranty of
 *  MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE.  See the
 *  GNU Lesser General Public License for more details.
 *
 *  You should have received a copy of the GNU Lesser General Public License
 *  along with this program.  If not, see <http://www.gnu.org/licenses/>.
 *
 */
//! This module contains the [`DoublePriorityQueue`] type and the related iterators.
//!
//! See the type level documentation for more details and examples.

use crate::{Allocator, Global};

pub mod iterators;

use indexmap::Vec;

use crate::core_iterators::{IntoIter, Iter};
use crate::store::Store;
use iterators::*;

use std::borrow::Borrow;
use std::cmp::{Eq, Ord};
#[cfg(has_std)]
use std::collections::hash_map::RandomState;
use std::hash::{BuildHasher, Hash};
use std::iter::{Extend, FromIterator, IntoIterator, Iterator};
use std::mem::replace;

/// A double priority queue with efficient change function to change the priority of an
/// element.
///
/// The priority is of type P, that must implement `std::cmp::Ord`.
///
/// The item is of type I, that must implement `Hash` and `Eq`.
///
/// Implemented as a heap of indexes, stores the items inside an `IndexMap`
/// to be able to retrieve them quickly.
///
/// With this data structure it is possible to efficiently extract both
/// the maximum and minimum elements arbitrarily.
///
/// If your need is to always extract the minimum, use a
/// `PriorityQueue<I, Reverse<P>>` wrapping
/// your priorities in the standard wrapper
/// [`Reverse<T>`](https://doc.rust-lang.org/std/cmp/struct.Reverse.html).
///
///
/// # Example
/// ```rust
/// use priority_queue::DoublePriorityQueue;
///
/// let mut pq = DoublePriorityQueue::new();
///
/// assert!(pq.is_empty());
/// pq.push("Apples", 5);
/// pq.push("Bananas", 8);
/// pq.push("Strawberries", 23);
///
/// assert_eq!(pq.peek_max(), Some((&"Strawberries", &23)));
/// assert_eq!(pq.peek_min(), Some((&"Apples", &5)));
///
/// pq.change_priority("Bananas", 25);
/// assert_eq!(pq.peek_max(), Some((&"Bananas", &25)));
///
/// for (item, _) in pq.into_sorted_iter() {
///     println!("{}", item);
/// }
/// ```
#[derive(Clone)]
#[cfg(has_std)]
pub struct DoublePriorityQueue<I, P, Arena = Global, H = RandomState>
where
    I: Hash + Eq,
    P: Ord,
    Arena: Allocator + Clone,
{
    pub(crate) store: Store<I, P, Arena, H>,
}

#[derive(Clone)]
#[cfg(not(has_std))]
pub struct DoublePriorityQueue<I, P, Arena = Global, H>
where
    I: Hash + Eq,
    P: Ord,
    Arena: Allocator + Clone,
{
    pub(crate) store: Store<I, P, Arena, H>,
}

// do not [derive(Eq)] to loosen up trait requirements for other types and impls
impl<I, P, Arena, H> Eq for DoublePriorityQueue<I, P, Arena, H>
where
    I: Hash + Eq,
    P: Ord,
    H: BuildHasher,
    Arena: Allocator + Clone,
{
}

impl<I, P, H> Default for DoublePriorityQueue<I, P, Global, H>
where
    I: Hash + Eq,
    P: Ord,
    H: BuildHasher + Default,
{
    fn default() -> Self {
        Self::with_default_hasher()
    }
}

#[cfg(has_std)]
impl<I, P> DoublePriorityQueue<I, P>
where
    P: Ord,
    I: Hash + Eq,
{
    /// Creates an empty `DoublePriorityQueue`
    pub fn new() -> Self {
        Self::with_capacity(0)
    }

    /// Creates an empty `DoublePriorityQueue` with the specified capacity.
    pub fn with_capacity(capacity: usize) -> Self {
        Self::with_capacity_and_default_hasher(capacity)
    }
}

#[cfg(has_std)]
impl<I, P, Arena> DoublePriorityQueue<I, P, Arena>
where
    P: Ord,
    I: Hash + Eq,
    Arena: Allocator + Clone,
{
    /// Creates an empty `DoublePriorityQueue`
    pub fn new_in(arena: Arena) -> Self {
        Self::with_capacity_in(0, arena)
    }

    /// Creates an empty `DoublePriorityQueue` with the specified capacity.
    pub fn with_capacity_in(capacity: usize, arena: Arena) -> Self {
        Self::with_capacity_and_default_hasher_in(capacity, arena)
    }
}

impl<I, P, H> DoublePriorityQueue<I, P, Global, H>
where
    P: Ord,
    I: Hash + Eq,
    H: BuildHasher + Default,
{
    /// Creates an empty `DoublePriorityQueue` with the default hasher
    pub fn with_default_hasher() -> Self {
        Self::with_capacity_and_default_hasher(0)
    }

    /// Creates an empty `DoublePriorityQueue` with the specified capacity and default hasher
    pub fn with_capacity_and_default_hasher(capacity: usize) -> Self {
        Self::with_capacity_and_hasher(capacity, H::default())
    }
}

impl<I, P, Arena, H> DoublePriorityQueue<I, P, Arena, H>
where
    P: Ord,
    I: Hash + Eq,
    H: BuildHasher + Default,
    Arena: Allocator + Clone,
{
    /// Creates an empty `DoublePriorityQueue` with the default hasher
    pub fn with_default_hasher_in(arena: Arena) -> Self {
        Self::with_capacity_and_default_hasher_in(0, arena)
    }

    /// Creates an empty `DoublePriorityQueue` with the specified capacity and default hasher
    pub fn with_capacity_and_default_hasher_in(capacity: usize, arena: Arena) -> Self {
        Self::with_capacity_and_hasher_in(capacity, H::default(), arena)
    }
}

impl<I, P, H> DoublePriorityQueue<I, P, Global, H>
where
    P: Ord,
    I: Hash + Eq,
    H: BuildHasher,
{
    /// Creates an empty `DoublePriorityQueue` with the specified hasher
    pub fn with_hasher(hash_builder: H) -> Self {
        Self::with_capacity_and_hasher(0, hash_builder)
    }

    /// Creates an empty `DoublePriorityQueue` with the specified capacity and hasher
    ///
    /// The internal collections will be able to hold at least `capacity`
    /// elements without reallocating.
    /// If `capacity` is 0, there will be no allocation.
    pub fn with_capacity_and_hasher(capacity: usize, hash_builder: H) -> Self {
        Self {
            store: Store::with_capacity_and_hasher(capacity, hash_builder),
        }
    }
}

impl<I, P, Arena, H> DoublePriorityQueue<I, P, Arena, H>
where
    P: Ord,
    I: Hash + Eq,
    H: BuildHasher,
    Arena: Allocator + Clone,
{
    /// Creates an empty `DoublePriorityQueue` with the specified hasher
    pub fn with_hasher_in(hash_builder: H, arena: Arena) -> Self {
        Self::with_capacity_and_hasher_in(0, hash_builder, arena)
    }

    /// Creates an empty `DoublePriorityQueue` with the specified capacity and hasher
    ///
    /// The internal collections will be able to hold at least `capacity`
    /// elements without reallocating.
    /// If `capacity` is 0, there will be no allocation.
    pub fn with_capacity_and_hasher_in(capacity: usize, hash_builder: H, arena: Arena) -> Self {
        Self {
            store: Store::with_capacity_and_hasher_in(capacity, hash_builder, arena),
        }
    }

    /// Returns an iterator in arbitrary order over the
    /// (item, priority) elements in the queue
    pub fn iter(&self) -> Iter<I, P> {
        self.store.iter()
    }
}

impl<I, P, Arena, H> DoublePriorityQueue<I, P, Arena, H>
where
    P: Ord,
    I: Hash + Eq,
    Arena: Allocator + Clone,
{
    /// Return an iterator in arbitrary order over the
    /// (item, priority) elements in the queue.
    ///
    /// The item and the priority are mutable references, but it's a logic error
    /// to modify the item in a way that change the result of `Hash` or `Eq`.
    ///
    /// It's *not* an error, instead, to modify the priorities, because the heap
    /// will be rebuilt once the `IterMut` goes out of scope. It would be
    /// rebuilt even if no priority value would have been modified, but the
    /// procedure will not move anything, but just compare the priorities.
    pub fn iter_mut(&mut self) -> IterMut<I, P, Arena, H> {
        IterMut::new(self)
    }

    /// Returns the couple (item, priority) with the lowest
    /// priority in the queue, or None if it is empty.
    ///
    /// Computes in **O(1)** time
    pub fn peek_min(&self) -> Option<(&I, &P)> {
        self.find_min().and_then(|i| {
            self.store
                .map
                .get_index(unsafe { *self.store.heap.get_unchecked(i) })
        })
    }

    /// Returns the couple (item, priority) with the greatest
    /// priority in the queue, or None if it is empty.
    ///
    /// Computes in **O(1)** time
    pub fn peek_max(&self) -> Option<(&I, &P)> {
        self.find_max().and_then(|i| {
            self.store
                .map
                .get_index(unsafe { *self.store.heap.get_unchecked(i) })
        })
    }

    /// Returns the couple (item, priority) with the lowest
    /// priority in the queue, or None if it is empty.
    ///
    /// The item is a mutable reference, but it's a logic error to modify it
    /// in a way that change the result of  `Hash` or `Eq`.
    ///
    /// The priority cannot be modified with a call to this function.
    /// To modify the priority use `push`, `change_priority` or
    /// `change_priority_by`.
    ///
    /// Computes in **O(1)** time
    pub fn peek_min_mut(&mut self) -> Option<(&mut I, &P)> {
        self.find_min()
            .and_then(move |i| {
                self.store
                    .map
                    .get_index_mut(unsafe { *self.store.heap.get_unchecked(i) })
            })
            .map(|(k, v)| (k, &*v))
    }

    /// Returns the couple (item, priority) with the greatest
    /// priority in the queue, or None if it is empty.
    ///
    /// The item is a mutable reference, but it's a logic error to modify it
    /// in a way that change the result of  `Hash` or `Eq`.
    ///
    /// The priority cannot be modified with a call to this function.
    /// To modify the priority use `push`, `change_priority` or
    /// `change_priority_by`.
    ///
    /// Computes in **O(1)** time
    pub fn peek_max_mut(&mut self) -> Option<(&mut I, &P)> {
        self.find_max()
            .and_then(move |i| {
                self.store
                    .map
                    .get_index_mut(unsafe { *self.store.heap.get_unchecked(i) })
            })
            .map(|(k, v)| (k, &*v))
    }

    /// Returns the number of elements the internal map can hold without
    /// reallocating.
    ///
    /// This number is a lower bound; the map might be able to hold more,
    /// but is guaranteed to be able to hold at least this many.
    pub fn capacity(&self) -> usize {
        self.store.capacity()
    }

    /// Shrinks the capacity of the internal data structures
    /// that support this operation as much as possible.
    pub fn shrink_to_fit(&mut self) {
        self.store.shrink_to_fit();
    }

    /// Removes the item with the lowest priority from
    /// the priority queue and returns the pair (item, priority),
    /// or None if the queue is empty.
    pub fn pop_min(&mut self) -> Option<(I, P)> {
        self.find_min().and_then(|i| {
            let r = self.store.swap_remove(i);
            self.heapify(i);
            r
        })
    }

    /// Removes the item with the greatest priority from
    /// the priority queue and returns the pair (item, priority),
    /// or None if the queue is empty.
    pub fn pop_max(&mut self) -> Option<(I, P)> {
        self.find_max().and_then(|i| {
            let r = self.store.swap_remove(i);
            self.heapify(i);
            r
        })
    }

    /// Implements a HeapSort.
    ///
    /// Consumes the PriorityQueue and returns a vector
    /// with all the items sorted from the one associated to
    /// the lowest priority to the highest.
    pub fn into_ascending_sorted_vec(mut self) -> Vec<I> {
        let mut res = Vec::with_capacity(self.store.size);
        while let Some((i, _)) = self.pop_min() {
            res.push(i);
        }
        res
    }

    /// Implements a HeapSort
    ///
    /// Consumes the PriorityQueue and returns a vector
    /// with all the items sorted from the one associated to
    /// the highest priority to the lowest.
    pub fn into_descending_sorted_vec(mut self) -> Vec<I> {
        let mut res = Vec::with_capacity(self.store.size);
        while let Some((i, _)) = self.pop_max() {
            res.push(i);
        }
        res
    }

    /// Returns the number of elements in the priority queue.
    pub fn len(&self) -> usize {
        self.store.len()
    }

    /// Returns true if the priority queue contains no elements.
    pub fn is_empty(&self) -> bool {
        self.store.is_empty()
    }

    /// Generates a new double ended iterator from self that
    /// will extract the elements from the one with the lowest priority
    /// to the highest one.
    pub fn into_sorted_iter(self) -> IntoSortedIter<I, P, Arena, H> {
        IntoSortedIter { pq: self }
    }
}

impl<I, P, Arena, H> DoublePriorityQueue<I, P, Arena, H>
where
    P: Ord,
    I: Hash + Eq,
    H: BuildHasher,
    Arena: Allocator + Clone,
{
    // reserve_exact -> IndexMap does not implement reserve_exact

    /// Reserves capacity for at least `additional` more elements to be inserted
    /// in the given `DoublePriorityQueue`. The collection may reserve more space to avoid
    /// frequent reallocations. After calling `reserve`, capacity will be
    /// greater than or equal to `self.len() + additional`. Does nothing if
    /// capacity is already sufficient.
    ///
    /// # Panics
    ///
    /// Panics if the new capacity overflows `usize`.
    pub fn reserve(&mut self, additional: usize) {
        self.store.reserve(additional);
    }

    /// Insert the item-priority pair into the queue.
    ///
    /// If an element equal to `item` was already into the queue,
    /// it is updated and the old value of its priority is returned in `Some`;
    /// otherwise, returns `None`.
    ///
    /// Computes in **O(log(N))** time.
    pub fn push(&mut self, item: I, priority: P) -> Option<P> {
        use indexmap::map::Entry::*;
        let mut pos = 0;
        let mut oldp = None;

        match self.store.map.entry(item) {
            Occupied(mut e) => {
                oldp = Some(replace(e.get_mut(), priority));
                pos = unsafe { *self.store.qp.get_unchecked(e.index()) };
            }
            Vacant(e) => {
                e.insert(priority);
            }
        }

        if oldp.is_some() {
            self.up_heapify(pos);
            return oldp;
        }
        // get a reference to the priority
        // copy the current size of the heap
        let i = self.store.size;
        // add the new element in the qp vector as the last in the heap
        self.store.qp.push(i);
        self.store.heap.push(i);
        self.bubble_up(i, i);
        self.store.size += 1;
        None
    }

    /// Increase the priority of an existing item in the queue, or
    /// insert it if not present.
    ///
    /// If an element equal to `item` is already in the queue with a
    /// lower priority, its priority is increased to the new one
    /// without replacing the element and the old priority is returned.
    /// Otherwise, the new element is inserted into the queue.
    ///
    /// Returns `Some` if an element equal to `item` is already in the
    /// queue. If its priority is higher then `priority`, the latter is returned back,
    /// otherwise, the old priority is contained in the Option.
    /// If the item is not in the queue, `None` is returned.
    ///
    /// Computes in **O(log(N))** time.
    pub fn push_increase(&mut self, item: I, priority: P) -> Option<P> {
        if self.get_priority(&item).map_or(true, |p| priority > *p) {
            self.push(item, priority)
        } else {
            Some(priority)
        }
    }

    /// Decrease the priority of an existing item in the queue, or
    /// insert it if not present.
    ///
    /// If an element equal to `item` is already in the queue with a
    /// higher priority, its priority is decreased to the new one
    /// without replacing the element and the old priority is returned.
    /// Otherwise, the new element is inserted into the queue.
    ///
    /// Returns `Some` if an element equal to `item` is already in the
    /// queue. If its priority is lower then `priority`, the latter is returned back,
    /// otherwise, the old priority is contained in the Option.
    /// If the item is not in the queue, `None` is returned.
    ///
    /// Computes in **O(log(N))** time.
    pub fn push_decrease(&mut self, item: I, priority: P) -> Option<P> {
        if self.get_priority(&item).map_or(true, |p| priority < *p) {
            self.push(item, priority)
        } else {
            Some(priority)
        }
    }

    /// Change the priority of an Item returning the old value of priority,
    /// or `None` if the item wasn't in the queue.
    ///
    /// The argument `item` is only used for lookup, and is not used to overwrite the item's data
    /// in the priority queue.
    ///
    /// The item is found in **O(1)** thanks to the hash table.
    /// The operation is performed in **O(log(N))** time.
    pub fn change_priority<Q: ?Sized>(&mut self, item: &Q, new_priority: P) -> Option<P>
    where
        I: Borrow<Q>,
        Q: Eq + Hash,
    {
        if let Some((r, pos)) = self.store.change_priority(item, new_priority) {
            self.up_heapify(pos);
            Some(r)
        } else {
            None
        }
    }

    /// Change the priority of an Item using the provided function.
    /// The item is found in **O(1)** thanks to the hash table.
    /// The operation is performed in **O(log(N))** time (worst case).
    pub fn change_priority_by<Q: ?Sized, F>(&mut self, item: &Q, priority_setter: F)
    where
        I: Borrow<Q>,
        Q: Eq + Hash,
        F: FnOnce(&mut P),
    {
        if let Some(pos) = self.store.change_priority_by(item, priority_setter) {
            self.up_heapify(pos);
        }
    }

    /// Get the priority of an item, or `None`, if the item is not in the queue
    pub fn get_priority<Q: ?Sized>(&self, item: &Q) -> Option<&P>
    where
        I: Borrow<Q>,
        Q: Eq + Hash,
    {
        self.store.get_priority(item)
    }

    /// Get the couple (item, priority) of an arbitrary element, as reference
    /// or `None` if the item is not in the queue.
    pub fn get<Q: ?Sized>(&self, item: &Q) -> Option<(&I, &P)>
    where
        I: Borrow<Q>,
        Q: Eq + Hash,
    {
        self.store.get(item)
    }

    /// Get the couple (item, priority) of an arbitrary element, or `None`
    /// if the item was not in the queue.
    ///
    /// The item is a mutable reference, but it's a logic error to modify it
    /// in a way that change the result of  `Hash` or `Eq`.
    ///
    /// The priority cannot be modified with a call to this function.
    /// To modify the priority use `push`, `change_priority` or
    /// `change_priority_by`.
    pub fn get_mut<Q: ?Sized>(&mut self, item: &Q) -> Option<(&mut I, &P)>
    where
        I: Borrow<Q>,
        Q: Eq + Hash,
    {
        self.store.get_mut(item)
    }

    /// Remove an arbitrary element from the priority queue.
    /// Returns the (item, priority) couple or None if the item
    /// is not found in the queue.
    ///
    /// The operation is performed in **O(log(N))** time (worst case).
    pub fn remove<Q: ?Sized>(&mut self, item: &Q) -> Option<(I, P)>
    where
        I: Borrow<Q>,
        Q: Eq + Hash,
    {
        self.store.remove(item).map(|(item, priority, pos)| {
            if pos < self.store.size {
                self.up_heapify(pos);
            }

            (item, priority)
        })
    }

    /// Returns the items not ordered
    pub fn into_vec(self) -> Vec<I> {
        self.store.into_vec()
    }

    /// Drops all items from the priority queue
    pub fn clear(&mut self) {
        self.store.clear();
    }

    /// Move all items of the `other` queue to `self`
    /// ignoring the items Eq to elements already in `self`
    /// At the end, `other` will be empty.
    ///
    /// **Note** that at the end, the priority of the duplicated elements
    /// inside self may be the one of the elements in other,
    /// if other is longer than self
    pub fn append(&mut self, other: &mut Self) {
        self.store.append(&mut other.store);
        self.heap_build();
    }
}

impl<I, P, Arena, H> DoublePriorityQueue<I, P, Arena, H>
where
    P: Ord,
    I: Hash + Eq,
    Arena: Allocator + Clone,
{
}

impl<I, P, Arena, H> DoublePriorityQueue<I, P, Arena, H>
where
    P: Ord,
    I: Hash + Eq,
    Arena: Allocator + Clone,
{
    /**************************************************************************/
    /*                            internal functions                          */

    fn heapify(&mut self, i: usize) {
        if self.store.size <= 1 {
            return;
        }
        if level(i) % 2 == 0 {
            self.heapify_min(i)
        } else {
            self.heapify_max(i)
        }
    }

    fn heapify_min(&mut self, mut i: usize) {
        while i <= parent(self.store.size - 1) {
            let m = i;

            // Minimum of childs and grandchilds
            i = *[
                left(i),
                right(i),
                left(left(i)),
                right(left(i)),
                left(right(i)),
                right(right(i)),
            ]
            .iter()
            .filter_map(|i| self.store.heap.get(*i).map(|index| (i, index)))
            .map(|(i, index)| {
                self.store
                    .map
                    .get_index(*index)
                    .map(|(item, priority)| (i, item, priority))
                    .unwrap()
            })
            .min_by_key(|(_, _, priority)| *priority)
            .unwrap()
            .0;

            if unsafe {
                self.store.get_priority_from_heap_index(i)
                    < self.store.get_priority_from_heap_index(m)
            } {
                self.store.swap(i, m);
                if i > right(m) // i is a grandchild of m
                    && unsafe {
                        self.store.get_priority_from_heap_index(i)
                            > self.store.get_priority_from_heap_index(parent(i))
                    }
                {
                    self.store.swap(i, parent(i));
                }
            } else {
                break;
            }
        }
    }

    fn heapify_max(&mut self, mut i: usize) {
        while i <= parent(self.store.size - 1) {
            let m = i;

            // Minimum of childs and grandchilds
            i = *[
                left(i),
                right(i),
                left(left(i)),
                right(left(i)),
                left(right(i)),
                right(right(i)),
            ]
            .iter()
            .filter_map(|i| self.store.heap.get(*i).map(|index| (i, index)))
            .map(|(i, index)| {
                self.store
                    .map
                    .get_index(*index)
                    .map(|(item, priority)| (i, item, priority))
                    .unwrap()
            })
            .max_by_key(|(_, _, priority)| *priority)
            .unwrap()
            .0;

            if unsafe {
                self.store.get_priority_from_heap_index(i)
                    > self.store.get_priority_from_heap_index(m)
            } {
                self.store.swap(i, m);
                if i > right(m) // i is a grandchild of m
                    && unsafe {
                        self.store.get_priority_from_heap_index(i)
                            < self.store.get_priority_from_heap_index(parent(i))
                    }
                {
                    self.store.swap(i, parent(i));
                }
            } else {
                break;
            }
        }
    }

    fn bubble_up(&mut self, mut position: usize, map_position: usize) -> usize {
        if position > 0 {
            position = if level(position) % 2 == 0 {
                //on a min level
                if self
                    .store
                    .map
                    .get_index(unsafe { *self.store.heap.get_unchecked(parent(position)) })
                    .unwrap()
                    .1
                    < self.store.map.get_index(map_position).unwrap().1
                {
                    // and greater then parent
                    unsafe {
                        *self.store.heap.get_unchecked_mut(position) =
                            *self.store.heap.get_unchecked(parent(position));
                        *self
                            .store
                            .qp
                            .get_unchecked_mut(*self.store.heap.get_unchecked(position)) = position;
                    }
                    self.bubble_up_max(parent(position), map_position)
                } else {
                    // and less then parent
                    self.bubble_up_min(position, map_position)
                }
            } else if self
                .store
                .map
                .get_index(unsafe { *self.store.heap.get_unchecked(parent(position)) })
                .unwrap()
                .1
                > self.store.map.get_index(map_position).unwrap().1
            {
                // on a max level and less then parent
                unsafe {
                    *self.store.heap.get_unchecked_mut(position) =
                        *self.store.heap.get_unchecked(parent(position));
                    *self
                        .store
                        .qp
                        .get_unchecked_mut(*self.store.heap.get_unchecked(position)) = position;
                }
                self.bubble_up_min(parent(position), map_position)
            } else {
                // on a max level and greater then parent
                self.bubble_up_max(position, map_position)
            }
        }

        unsafe {
            // put the new element into the heap and
            // update the qp translation table and the size
            *self.store.heap.get_unchecked_mut(position) = map_position;
            *self.store.qp.get_unchecked_mut(map_position) = position;
        }
        position
    }

    fn bubble_up_min(&mut self, mut position: usize, map_position: usize) -> usize {
        while (position > 0 && parent(position) > 0)
            && (self
                .store
                .map
                .get_index(unsafe { *self.store.heap.get_unchecked(parent(parent(position))) })
                .unwrap()
                .1
                > self.store.map.get_index(map_position).unwrap().1)
        {
            unsafe {
                *self.store.heap.get_unchecked_mut(position) =
                    *self.store.heap.get_unchecked(parent(parent(position)));
                *self
                    .store
                    .qp
                    .get_unchecked_mut(*self.store.heap.get_unchecked(position)) = position;
            }
            position = parent(parent(position));
        }
        position
    }

    fn bubble_up_max(&mut self, mut position: usize, map_position: usize) -> usize {
        while (position > 0 && parent(position) > 0)
            && (self
                .store
                .map
                .get_index(unsafe { *self.store.heap.get_unchecked(parent(parent(position))) })
                .unwrap()
                .1
                < self.store.map.get_index(map_position).unwrap().1)
        {
            unsafe {
                *self.store.heap.get_unchecked_mut(position) =
                    *self.store.heap.get_unchecked(parent(parent(position)));
                *self
                    .store
                    .qp
                    .get_unchecked_mut(*self.store.heap.get_unchecked(position)) = position;
            }
            position = parent(parent(position));
        }
        position
    }

    fn up_heapify(&mut self, i: usize) {
        let tmp = unsafe { *self.store.heap.get_unchecked(i) };
        let pos = self.bubble_up(i, tmp);
        self.heapify(pos);
    }

    /// Internal function that transform the `heap`
    /// vector in a heap with its properties
    ///
    /// Computes in **O(N)**
    pub(crate) fn heap_build(&mut self) {
        if self.store.size == 0 {
            return;
        }
        for i in (0..=parent(self.store.size)).rev() {
            self.heapify(i);
        }
    }

    /// Returns the index of the max element
    fn find_max(&self) -> Option<usize> {
        match self.store.size {
            0 => None,
            1 => Some(0),
            2 => Some(1),
            _ => Some(
                *[1, 2]
                    .iter()
                    .max_by_key(|i| unsafe { self.store.get_priority_from_heap_index(**i) })
                    .unwrap(),
            ),
        }
    }

    /// Returns the index of the min element
    fn find_min(&self) -> Option<usize> {
        match self.store.size {
            0 => None,
            _ => Some(0),
        }
    }
}

//FIXME: fails when the vector contains repeated items
// FIXED: repeated items ignored
impl<I, P, H> From<std::vec::Vec<(I, P)>> for DoublePriorityQueue<I, P, Global, H>
where
    I: Hash + Eq,
    P: Ord,
    H: BuildHasher + Default,
{
    fn from(vec: std::vec::Vec<(I, P)>) -> Self {
        let store = Store::<I, P, Global, H>::from(vec);
        let mut pq = DoublePriorityQueue { store };
        pq.heap_build();
        pq
    }
}

use crate::PriorityQueue;

impl<I, P, H> From<PriorityQueue<I, P, Global, H>> for DoublePriorityQueue<I, P, Global, H>
where
    I: Hash + Eq,
    P: Ord,
    H: BuildHasher,
{
    fn from(pq: PriorityQueue<I, P, Global, H>) -> Self {
        let store = pq.store;
        let mut this = Self { store };
        this.heap_build();
        this
    }
}

//FIXME: fails when the iterator contains repeated items
// FIXED: the item inside the pq is updated
// so there are two functions with different behaviours.
impl<I, P, H> FromIterator<(I, P)> for DoublePriorityQueue<I, P, Global, H>
where
    I: Hash + Eq,
    P: Ord,
    H: BuildHasher + Default,
{
    fn from_iter<IT>(iter: IT) -> Self
    where
        IT: IntoIterator<Item = (I, P)>,
    {
        let store = Store::from_iter(iter);
        let mut pq = DoublePriorityQueue { store };
        pq.heap_build();
        pq
    }
}

impl<I, P, Arena, H> IntoIterator for DoublePriorityQueue<I, P, Arena, H>
where
    I: Hash + Eq,
    P: Ord,
    H: BuildHasher,
    Arena: Allocator + Clone,
{
    type Item = (I, P);
    type IntoIter = IntoIter<I, P, Arena>;
    fn into_iter(self) -> IntoIter<I, P, Arena> {
        self.store.into_iter()
    }
}

impl<'a, I, P, Arena, H> IntoIterator for &'a DoublePriorityQueue<I, P, Arena, H>
where
    I: Hash + Eq,
    P: Ord,
    H: BuildHasher,
    Arena: Allocator + Clone,
{
    type Item = (&'a I, &'a P);
    type IntoIter = Iter<'a, I, P>;
    fn into_iter(self) -> Iter<'a, I, P> {
        self.store.iter()
    }
}

impl<'a, I, P, Arena, H> IntoIterator for &'a mut DoublePriorityQueue<I, P, Arena, H>
where
    I: Hash + Eq,
    P: Ord,
    Arena: Allocator + Clone,
{
    type Item = (&'a mut I, &'a mut P);
    type IntoIter = IterMut<'a, I, P, Arena, H>;
    fn into_iter(self) -> IterMut<'a, I, P, Arena, H> {
        IterMut::new(self)
    }
}

impl<I, P, Arena, H> Extend<(I, P)> for DoublePriorityQueue<I, P, Arena, H>
where
    I: Hash + Eq,
    P: Ord,
    H: BuildHasher,
    Arena: Allocator + Clone,
{
    fn extend<T: IntoIterator<Item = (I, P)>>(&mut self, iter: T) {
        let iter = iter.into_iter();
        let (min, max) = iter.size_hint();
        let rebuild = if let Some(max) = max {
            self.reserve(max);
            better_to_rebuild(self.store.size, max)
        } else if min != 0 {
            self.reserve(min);
            better_to_rebuild(self.store.size, min)
        } else {
            false
        };
        if rebuild {
            self.store.extend(iter);
            self.heap_build();
        } else {
            for (item, priority) in iter {
                self.push(item, priority);
            }
        }
    }
}

use std::fmt;

impl<I, P, Arena, H> fmt::Debug for DoublePriorityQueue<I, P, Arena, H>
where
    I: Hash + Eq + fmt::Debug,
    P: Ord + fmt::Debug,
    Arena: Allocator + Clone,
{
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> Result<(), fmt::Error> {
        self.store.fmt(f)
    }
}

use std::cmp::PartialEq;

impl<I, P1, H1, P2, H2, Arena> PartialEq<DoublePriorityQueue<I, P2, Arena, H2>>
    for DoublePriorityQueue<I, P1, Arena, H1>
where
    I: Hash + Eq,
    P1: Ord,
    P1: PartialEq<P2>,
    Option<P1>: PartialEq<Option<P2>>,
    P2: Ord,
    H1: BuildHasher,
    H2: BuildHasher,
    Arena: Allocator + Clone,
{
    fn eq(&self, other: &DoublePriorityQueue<I, P2, Arena, H2>) -> bool {
        self.store == other.store
    }
}

/// Compute the index of the left child of an item from its index
fn left(i: usize) -> usize {
    (i * 2) + 1
}
/// Compute the index of the right child of an item from its index
fn right(i: usize) -> usize {
    (i * 2) + 2
}
/// Compute the index of the parent element in the heap from its index
fn parent(i: usize) -> usize {
    (i - 1) / 2
}

// Compute the level of a node from its index
fn level(i: usize) -> usize {
    log2_fast(i + 1)
}

fn log2_fast(x: usize) -> usize {
    use std::mem::size_of;

    8 * size_of::<usize>() - (x.leading_zeros() as usize) - 1
}

// `rebuild` takes O(len1 + len2) operations
// and about 2 * (len1 + len2) comparisons in the worst case
// while `extend` takes O(len2 * log_2(len1)) operations
// and about 1 * len2 * log_2(len1) comparisons in the worst case,
// assuming len1 >= len2.
fn better_to_rebuild(len1: usize, len2: usize) -> bool {
    // log(1) == 0, so the inequation always falsy
    // log(0) is inapplicable and produces panic
    if len1 <= 1 {
        return false;
    }

    2 * (len1 + len2) < len2 * log2_fast(len1)
}

#[cfg(feature = "serde")]
mod serde {
    use std::cmp::{Eq, Ord};
    use std::hash::{BuildHasher, Hash};

    use serde::de::{Deserialize, Deserializer};
    use serde::ser::{Serialize, Serializer};

    use super::DoublePriorityQueue;
    use crate::store::Store;

    impl<I, P, Arena, H> Serialize for DoublePriorityQueue<I, P, Arena, H>
    where
        I: Hash + Eq + Serialize,
        P: Ord + Serialize,
        H: BuildHasher,
        Arena: Allocator + Clone,
    {
        fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
        where
            S: Serializer,
        {
            self.store.serialize(serializer)
        }
    }

    impl<'de, I, P, Arena, H> Deserialize<'de> for DoublePriorityQueue<I, P, Arena, H>
    where
        I: Hash + Eq + Deserialize<'de>,
        P: Ord + Deserialize<'de>,
        H: BuildHasher + Default,
        Arena: Allocator + Clone,
    {
        fn deserialize<D>(deserializer: D) -> Result<DoublePriorityQueue<I, P, H>, D::Error>
        where
            D: Deserializer<'de>,
        {
            Store::deserialize(deserializer).map(|store| {
                let mut pq = DoublePriorityQueue { store };
                pq.heap_build();
                pq
            })
        }
    }
}
