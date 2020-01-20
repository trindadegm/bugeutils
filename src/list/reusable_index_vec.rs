/* *****************************************************************************
 MIT License
 
 Copyright (c) 2020 trindadegm
 
 Permission is hereby granted, free of charge, to any person obtaining a copy
 of this software and associated documentation files (the "Software"), to deal
 in the Software without restriction, including without limitation the rights
 to use, copy, modify, merge, publish, distribute, sublicense, and/or sell
 copies of the Software, and to permit persons to whom the Software is
 furnished to do so, subject to the following conditions:
 
 The above copyright notice and this permission notice shall be included in all
 copies or substantial portions of the Software.
 
 THE SOFTWARE IS PROVIDED "AS IS", WITHOUT WARRANTY OF ANY KIND, EXPRESS OR
 IMPLIED, INCLUDING BUT NOT LIMITED TO THE WARRANTIES OF MERCHANTABILITY,
 FITNESS FOR A PARTICULAR PURPOSE AND NONINFRINGEMENT. IN NO EVENT SHALL THE
 AUTHORS OR COPYRIGHT HOLDERS BE LIABLE FOR ANY CLAIM, DAMAGES OR OTHER
 LIABILITY, WHETHER IN AN ACTION OF CONTRACT, TORT OR OTHERWISE, ARISING FROM,
 OUT OF OR IN CONNECTION WITH THE SOFTWARE OR THE USE OR OTHER DEALINGS IN THE
 SOFTWARE.
***************************************************************************** */
use crate::error::{Error as BugeError, ErrorType as BugeErrorType};

type ListResult<T> = Result<T, BugeError>;

pub type Index = usize;
pub type CycleStamp = u32;

#[derive(Debug, PartialEq, Eq)]
pub struct ID(pub CycleStamp, pub Index);

/// This enum elaborates which kind of nodes will exist inside of the vector.
/// 
/// For someone who is using this, probably by accessing the slice with all of the elements inside.
/// You most certainly is only worried about the `Exists` variation. `Removed` and `RemovedAndNext`
/// both mean that the value has been removed, but keep information for bookkeeping.
///
/// The `CycleStamp` is part of the `ID` of the element. The `ID` is a combination of the
/// `CycleStamp` and the `Index` that the element occupies on the vector. As an element may reuse
/// a position, the indices of the elements may be the same. To avoid this confusion, a
/// `CycleStamp` is used to differentiate between the new elements on that index from the old
/// elements on that same index.
///
/// The reason for this structure to be organized this way, as well for the `CycleStamp` having 32
/// bits. Is to make this `ReusableIndexNode` have only 8 bytes (64 bits) more than the size of `T`.
/// This is assuming the type `T` has been aligned to a 64 bit word. This is not an optimization on
/// 32 bit machines, but it will still work. It was done because I figured doing it in some other
/// ways was just very wasteful on memory, as there will be long vectors of this thing.
#[derive(Debug)]
pub enum ReusableIndexNode<T> {
    /// The value of type `T` exists. It is on the cycle `CycleStamp`.
    Exists(CycleStamp, T),
    /// The value has been removed.
    Removed(CycleStamp),
    /// The value has been removed. This is used for bookkeeping.
    RemovedAndNext(CycleStamp, Index),
}

#[derive(Debug)]
/// A fast implementation of a map-like data structure that assigns IDs for every added element.
///
/// ```
///     use bugeutils::list::ReusableIndexVec;
///
///     let mut entity_vec = ReusableIndexVec::new();
///
///     let string1_id = entity_vec.add("A string is added");
///     let string2_id = entity_vec.add("Another string with another id");
///
///     assert_ne!(string1_id, string2_id);
///
///     if let Some(some_string) = entity_vec.get(&string1_id) {
///         println!("{}", some_string); // prints 'A string is added'
///     }
/// ```
pub struct ReusableIndexVec<T> {
    vector: Vec<ReusableIndexNode<T>>,
    last_removed: Option<Index>,
}

const DEFAULT_INITIAL_CAPACITY: usize = 128;

impl<T> ReusableIndexVec<T> {
    #[inline]
    /// Creates a new empty `ReusableIndexVec`.
    pub fn new() -> Self {
        Self::with_capacity(DEFAULT_INITIAL_CAPACITY)
    }

    #[inline]
    /// Creates a new empty `ReusableIndexVec` with a given initial capacity. This does not feeds
    /// any elements on the struct, just pre-allocates them.
    pub fn with_capacity(capacity: usize) -> Self {
        Self {
            vector: Vec::with_capacity(capacity),
            last_removed: None,
        }
    }

    /// Adds a new element, returning a given ID associated with it.
    pub fn add(&mut self, node: T) -> ID {
        let new_cycle_stamp;
        let added_at_index;

        if let Some(last_removed) = self.last_removed {
            // A node has been removed before, let's use his place in his memory.
            debug_assert!(last_removed < self.vector.len(), "[LOGIC ERROR] Last removed index is out of bounds!");

            added_at_index = last_removed; // It will add the new one here

            let node_state = &self.vector[last_removed];

            match node_state {
                ReusableIndexNode::Removed(cycle_stamp) => {
                    new_cycle_stamp = cycle_stamp.wrapping_add(1);
                    self.vector[last_removed] = ReusableIndexNode::Exists(new_cycle_stamp, node);
                    self.last_removed = None;
                },
                ReusableIndexNode::RemovedAndNext(cycle_stamp, next_removed) => {
                    new_cycle_stamp = cycle_stamp.wrapping_add(1);
                    let next_removed = *next_removed; // Make a copy of the next_removed value, as it will be replaced...
                    self.vector[last_removed] = ReusableIndexNode::Exists(new_cycle_stamp, node); // ...on this line
                    self.last_removed = Some(next_removed);
                },
                // This should never actually execute. If it does, it is a bug.
                ReusableIndexNode::Exists(_, _) => panic!("[LOGIC ERROR] Node at {} should not exist", last_removed),
            }
        } else {
            // Creating a brand new node.
            new_cycle_stamp = 0;
            self.vector.push(ReusableIndexNode::Exists(new_cycle_stamp, node));
            added_at_index = self.vector.len() - 1;
        }

        ID(new_cycle_stamp, added_at_index)
    }

    /// Removes the element associated with the given ID.
    ///
    /// # Errors
    /// This function returns error of type `NotFound` if the element has never existed, or was removed.
    pub fn remove(&mut self, id: &ID) -> ListResult<()> {
        let (requested_cycle_stamp, index) = (id.0, id.1);

        if index < self.vector.len() {
            if let ReusableIndexNode::Exists(cycle_stamp, _) = self.vector[index] {
                if requested_cycle_stamp == cycle_stamp {
                    if let Some(last_removed) = self.last_removed {
                        self.vector[index] = ReusableIndexNode::RemovedAndNext(cycle_stamp, last_removed);
                    } else {
                        self.vector[index] = ReusableIndexNode::Removed(cycle_stamp);
                    }

                    self.last_removed = Some(index);

                    return Ok(())
                }
            }
        }

        Err(BugeError::new(BugeErrorType::NotFound, &format!("node with id {}::{} not found", requested_cycle_stamp, index)))
    }

    // Not used
    //fn remove_by_index(&mut self, index: Index) -> ListResult<()> {
    //    if index < self.vector.len() {
    //        if let ReusableIndexNode::Exists(cycle_stamp, _) = self.vector[index] {
    //            if let Some(last_removed) = self.last_removed {
    //                self.vector[index] = ReusableIndexNode::RemovedAndNext(cycle_stamp, last_removed);
    //            } else {
    //                self.vector[index] = ReusableIndexNode::Removed(cycle_stamp);
    //            }

    //            self.last_removed = Some(index);

    //            Ok(())
    //        } else {
    //            Err(BugeError::new(BugeErrorType::NotFound, &format!("node with index {} does not exist. It cannot be removed", index)))
    //        }
    //    } else {
    //        Err(BugeError::new(BugeErrorType::NotFound, &format!("index {} is out of bounds. It cannot be removed", index)))
    //    }
    //}

    fn get_by_index(&self, index: Index) -> Option<(CycleStamp, &T)> {
        if index < self.vector.len() {
            if let ReusableIndexNode::Exists(cycle_stamp, ref node) = self.vector[index] {
                Some((cycle_stamp, node))
            } else {
                None
            }
        } else {
            None
        }
    }

    fn get_by_index_mut(&mut self, index: Index) -> Option<(CycleStamp, &mut T)> {
        if index < self.vector.len() {
            if let ReusableIndexNode::Exists(cycle_stamp, ref mut node) = self.vector[index] {
                Some((cycle_stamp, node))
            } else {
                None
            }
        } else {
            None
        }
    }

    /// Returns a reference to the element associated with the given ID.
    ///
    /// Returns `None` if the element does not exist.
    pub fn get(&mut self, id: &ID) -> Option<&T> {
        let ID(cycle_stamp, index) = id;
        let (found_cycle_stamp, node) = self.get_by_index(*index)?;

        // If it is REALLY the same
        if *cycle_stamp == found_cycle_stamp {
            Some(node)
        } else {
            None
        }
    }

    /// Returns a mutable reference to the element associated with the given ID.
    ///
    /// Returns `None` if the element does not exist.
    pub fn get_mut(&mut self, id: &ID) -> Option<&mut T> {
        let ID(cycle_stamp, index) = id;
        let (found_cycle_stamp, node) = self.get_by_index_mut(*index)?;

        // If it is REALLY the same
        if *cycle_stamp == found_cycle_stamp {
            Some(node)
        } else {
            None
        }
    }

    /// Returns a slice to a list of nodes.
    #[inline]
    pub fn as_slice(&self) -> &[ReusableIndexNode<T>] {
        self.vector.as_slice()
    }
} // End of impl ReusableIndexVec

#[cfg(test)]
mod tests {
    #[test]
    fn size_test() {
        use std::mem;

        use crate::list::ReusableIndexNode;

        assert_eq!(mem::size_of::<ReusableIndexNode<u32>>(), 16);
        assert_eq!(mem::size_of::<ReusableIndexNode<u64>>(), 16);
        assert_eq!(mem::size_of::<ReusableIndexNode<u128>>(), 24);
    }

    #[test]
    fn test_creation() {
    }
}
