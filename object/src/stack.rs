use std::fmt::Debug;
use std::mem::MaybeUninit;
use std::sync::atomic::{AtomicPtr, AtomicUsize, Ordering};

/// scalable lock-free stack(Treiber Stack) designed for concurrency programing
///
/// The [`AtomicStack<C>`] operate dataflow in FILO.
#[derive(Debug)]
pub struct AtomicStack<C>
where
    C: Unpin,
{
    length: AtomicUsize,
    head: AtomicPtr<Node<C>>,
}

impl<C> AtomicStack<C>
where
    C: Unpin,
{
    /// Creates a new [`AtomicStack<C>`].
    #[inline]
    pub fn new() -> Self {
        // let node = Box::into_raw(Node::new(MaybeUninit::uninit()));
        // let head = AtomicPtr::new(node);
        let head = AtomicPtr::default();
        AtomicStack {
            length: AtomicUsize::new(0),
            head,
        }
    }
    /// Push a element into the stack
    #[inline]
    pub fn push(&self, element: C) {
        let node = Node::new(MaybeUninit::new(element));
        let node = Box::leak(node);

        loop {
            let head = self.head.load(Ordering::Relaxed);
            node.next.store(head, Ordering::Relaxed);

            if self
                .head
                .compare_exchange(head, node, Ordering::Release, Ordering::Relaxed)
                .is_ok()
            {
                self.length.fetch_add(1, Ordering::Relaxed);
                break;
            }
        }
    }
    /// Pop a element of the stack without checking whether the stack is empty or not
    ///
    /// Returns the wrapped element [`GC<C>] from the top of stack
    #[inline]
    pub unsafe fn unchecked_pop(&self) -> GC<C> {
        self.length.fetch_sub(1, Ordering::Relaxed);
        loop {
            let head = self.head.load(Ordering::Acquire);
            let dropped = unsafe { (*head).next.load(Ordering::Acquire) };
            if let Ok(head) =
                self.head
                    .compare_exchange(head, dropped, Ordering::Release, Ordering::Relaxed)
            {
                // let data = unsafe { Box::from_raw(head) };
                return GC { node: head };
            }
        }
    }
    /// Pop a element of the [`AtomicStack<C>`]
    ///
    /// Returns the option of wrapped element [`Option<GC<C>>]
    #[inline]
    pub fn pop(&self) -> Option<GC<C>> {
        self.length.fetch_sub(1, Ordering::Relaxed);
        loop {
            if self.head.load(Ordering::Acquire).is_null() {
                return None;
            }
            let head = self.head.load(Ordering::Acquire);
            let dropped = unsafe { (*head).next.load(Ordering::Acquire) };

            if let Ok(head) =
                self.head
                    .compare_exchange(head, dropped, Ordering::Release, Ordering::Relaxed)
            {
                // let data = unsafe { Box::from_raw(head) };
                return Some(GC { node: head });
            }
        }
    }
    /// Returns the length of this [`AtomicStack<C>`].
    #[inline]
    pub fn len(&self) -> usize {
        self.length.load(Ordering::Acquire)
    }
    /// Returns true if the [`AtomicStack<C>`] is empty
    #[inline]
    pub fn is_empty(&self) -> bool {
        let head = self.head.load(Ordering::Acquire);
        head.is_null()
    }
}

impl<C> Drop for AtomicStack<C>
where
    C: Unpin,
{
    fn drop(&mut self)
    where
        C: Unpin,
    {
        while AtomicStack::pop(&self).is_some() {}
    }
}

pub struct Node<C> {
    data: MaybeUninit<C>,
    next: AtomicPtr<Node<C>>,
}

impl<C> Node<C> {
    fn new(data: MaybeUninit<C>) -> Box<Self> {
        Box::new(Node {
            data,
            next: AtomicPtr::default(),
        })
    }
}

/// a wrapper for element pop for [`AtomicStack<C>`]
pub struct GC<T> {
    node: *mut Node<T>,
}

impl<T> AsMut<T> for GC<T> {
    fn as_mut(&mut self) -> &mut T {
        unsafe { (*self.node).data.assume_init_mut() }
    }
}

impl<T> AsRef<T> for GC<T> {
    fn as_ref(&self) -> &T {
        unsafe { (*self.node).data.assume_init_ref() }
    }
}

impl<T> Drop for GC<T> {
    fn drop(&mut self) {
        unsafe {
            drop(Box::from_raw(self.node));
        }
    }
}

#[cfg(test)]
mod test {
    use std::{
        sync::{atomic::Ordering, Arc},
        thread::{self, Thread},
    };

    use super::*;

    #[test]
    fn stack_is_empty() {
        let stack: AtomicStack<usize> = AtomicStack::new();
        assert!(stack.is_empty());
    }

    #[test]
    fn stack_unchecked_pop() {
        let stack = AtomicStack::new();
        stack.push(1001_usize);
        stack.push(1002_usize);
        stack.push(1003_usize);
        stack.push(1004_usize);
        unsafe {
            assert_eq!(*stack.unchecked_pop().as_ref(), 1004_usize);
            assert_eq!(*stack.unchecked_pop().as_ref(), 1003_usize);
            assert_eq!(*stack.unchecked_pop().as_ref(), 1002_usize);
            assert_eq!(*stack.unchecked_pop().as_ref(), 1001_usize);
        }
    }

    #[test]
    fn stack_pop() {
        let stack = AtomicStack::new();
        stack.push(1002_usize);
        stack.push(1003_usize);
        stack.push(1004_usize);
        assert_eq!(*stack.pop().unwrap().as_ref(), 1004_usize);
        assert_eq!(*stack.pop().unwrap().as_ref(), 1003_usize);
        assert_eq!(*stack.pop().unwrap().as_ref(), 1002_usize);
        assert!(stack.pop().as_ref().is_none());
    }

    #[test]
    fn stack_multi_threading() {
        let stack = AtomicStack::new();
        let stack = Box::leak(Box::new(stack));
        let mut threads = vec![];
        for _ in 0..10 {
            threads.push(thread::spawn(|| {
                for _ in 0..100 {
                    stack.push(1008_usize);
                    assert_eq!(1008_usize, *stack.pop().unwrap().as_ref())
                }
            }));
        }
    }
}
