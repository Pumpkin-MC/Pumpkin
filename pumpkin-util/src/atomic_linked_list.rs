use core::sync::atomic::{AtomicPtr, Ordering};
// use core::marker::PhantomData;

struct Node<T> {
    data: T,
    next: AtomicPtr<Node<T>>,
}
impl<T> Node<T> {
    fn new(data: T) -> Node<T> {
        Node {
            data,
            next: AtomicPtr::default(), // null ptr by default
        }
    }
}

#[derive(Debug)]
pub struct AtomicLinkedList<T> {
    head: AtomicPtr<Node<T>>,
}
impl<T: PartialEq> Default for AtomicLinkedList<T> {
    fn default() -> Self {
        Self::new()
    }
}
impl<T: PartialEq> AtomicLinkedList<T> {
    /// Create a new empty AtomicLinkedList.
    ///
    /// Does not perform any allocation until a new node is created.
    pub const fn new() -> AtomicLinkedList<T> {
        AtomicLinkedList {
            head: AtomicPtr::new(core::ptr::null_mut()), // null ptr
        }
    }

    /// add a new element to the front of the list.
    pub fn push_front(&self, data: T) -> Result<(), T> {
        self.push_front_timeout(data, u64::MAX)
    }

    pub fn remove(&self, value: &T) -> bool {
        let mut current = &self.head;
        let mut attempts = 0;
        const MAX_ATTEMPTS: u32 = 100; // Adjust as needed

        while attempts < MAX_ATTEMPTS {
            let current_ptr = current.load(Ordering::Acquire);
            if current_ptr.is_null() {
                return false; // Value not found
            }

            // SAFETY: We've checked that current_ptr is not null
            unsafe {
                let current_ref = &*current_ptr;
                if current_ref.data == *value {
                    // Found the value, try to remove it
                    let next_ptr = current_ref.next.load(Ordering::Acquire);
                    match current.compare_exchange(
                        current_ptr,
                        next_ptr,
                        Ordering::AcqRel,
                        Ordering::Acquire,
                    ) {
                        Ok(_) => {
                            // Successfully removed the node
                            // SAFETY: We own this node now and can safely deallocate it
                            drop(Box::from_raw(current_ptr));
                            return true;
                        }
                        Err(_) => {
                            // CAS failed, try again
                            attempts += 1;
                            continue;
                        }
                    }
                }
                // Move to the next node
                current = &current_ref.next;
            }
        }

        false // Exceeded maximum attempts
    }

    /// add a new element to the front of the list, but will abort
    /// if it fails to do so atomically after the given number of attempts.
    pub fn push_front_timeout(&self, data: T, max_attempts: u64) -> Result<(), T> {
        let max_attempts = core::cmp::max(max_attempts, 1); // ensure we try at least once

        let node_ptr = Box::into_raw(Box::new(Node::new(data)));

        // start the first attempt by obtaining the current head pointer
        let mut orig_head_ptr = self.head.load(Ordering::Acquire);
        for _attempt in 0..max_attempts {
            // the new "node" will become the new head, so set the node's `next` pointer to `orig_head_ptr`
            // SAFE: we know the node_ptr is valid since we just created it above.
            unsafe {
                (*node_ptr).next = AtomicPtr::new(orig_head_ptr);
            }

            // now try to atomically swap the new `node_ptr` into the current `head` ptr
            match self.head.compare_exchange_weak(
                orig_head_ptr,
                node_ptr,
                Ordering::AcqRel,
                Ordering::Acquire,
            ) {
                // If compare_exchange succeeds, then the `head` ptr was properly updated, i.e.,
                // no other thread was interleaved and snuck in to change `head` since we last loaded it.
                Ok(_old_head_ptr) => return Ok(()),
                Err(changed_head_ptr) => orig_head_ptr = changed_head_ptr,
            }

            // Here, it didn't work, the head value wasn't updated, meaning that another process updated it before we could
            // so we need to start over by reading the head ptr again and trying to swap it in again
            #[cfg(test)]
            println!("        attempt {}", _attempt);
        }

        // Here, we exceeded the number of max attempts, so we failed.
        // Reclaim the Boxed `Node`, drop the Box, and return the inner data of type `T`.
        // SAFE: no one has touched this node except for us when we created it above.
        let reclaimed_node = unsafe { Box::from_raw(node_ptr) };

        Err(reclaimed_node.data)
    }

    /// returns a forward iterator through this linked list.
    pub fn iter(&self) -> AtomicLinkedListIter<T> {
        AtomicLinkedListIter {
            curr: &self.head, //load(Ordering::Acquire),
                              // _phantom: PhantomData,
        }
    }

    /// returns a forward iterator through this linked list,
    /// allowing mutation of inner elements.
    pub fn iter_mut(&self) -> AtomicLinkedListIterMut<T> {
        AtomicLinkedListIterMut {
            curr: &self.head, //load(Ordering::Acquire),
                              // _phantom: PhantomData,
        }
    }
}

pub struct AtomicLinkedListIter<'a, T: 'a> {
    curr: &'a AtomicPtr<Node<T>>,
    // _phantom: PhantomData<&'a T>, // we don't need this with the &'a above
}
impl<'a, T: 'a> Iterator for AtomicLinkedListIter<'a, T> {
    type Item = &'a T;

    fn next(&mut self) -> Option<&'a T> {
        let curr_ptr = self.curr.load(Ordering::Acquire);
        if curr_ptr.is_null() {
            return None;
        }
        // SAFE: curr_ptr was checked for null
        let curr_node: &Node<T> = unsafe { &*curr_ptr };
        self.curr = &curr_node.next; // advance the iterator
        Some(&curr_node.data)
    }
}

pub struct AtomicLinkedListIterMut<'a, T: 'a> {
    curr: &'a AtomicPtr<Node<T>>,
    // _phantom: PhantomData<&'a T>, // we don't need this with the &'a above
}
impl<'a, T: 'a> Iterator for AtomicLinkedListIterMut<'a, T> {
    type Item = &'a mut T;

    fn next(&mut self) -> Option<&'a mut T> {
        let curr_ptr = self.curr.load(Ordering::Acquire);
        if curr_ptr.is_null() {
            return None;
        }
        // SAFE: curr_ptr was checked for null
        let curr_node: &mut Node<T> = unsafe { &mut *curr_ptr };
        self.curr = &curr_node.next; // advance the iterator
        Some(&mut curr_node.data)
    }
}
