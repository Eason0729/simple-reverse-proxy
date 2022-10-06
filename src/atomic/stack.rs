use std::fmt::Debug;
use std::sync::atomic::{AtomicPtr, AtomicUsize, Ordering};

use super::epoch::Global;
use super::epoch::Local;

const STACK_CAP: usize = 33;// >= (thread_count*2 + 1)
const STACK_LIMIT: usize = 8;// >= 2

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
    garbage_collector: Global<Node<C>, STACK_CAP>,
}

impl<C> AtomicStack<C>
where
    C: Unpin,
{
    /// Creates a new [`AtomicStack<C>`].
    pub fn new() -> Self {
        let global: Global<Node<C>, STACK_CAP> = Global::new();
        AtomicStack {
            length: AtomicUsize::new(0),
            head: AtomicPtr::default(),
            garbage_collector: global,
        }
    }
    /// Garbage collector(local) are used to prevent access to invaild ptr
    ///
    /// Returns the garbage collector(local) corresponded to this [`AtomicStack<C>`].
    pub fn get_gc(&self) -> Local<'_, Node<C>, STACK_LIMIT, STACK_CAP> {
        let mut local = Local::new(&self.garbage_collector);
        local.pin(); 
        local.unpin();
        local
    }
    /// Push a element into the stack
    pub fn push(&self, element: C, gc: &mut Local<'_, Node<C>, STACK_LIMIT, STACK_CAP>) {
        let node = Node::new(element);
        let node = Box::leak(node);

        loop {
            gc.pin();
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
            gc.unpin();
        }
    }
    /// Pop a element of the [`AtomicStack<C>`]
    ///
    /// Returns the option of wrapped element [`Option<Wrapper<C>>]
    pub fn pop(&self, gc: &mut Local<'_, Node<C>, STACK_LIMIT, STACK_CAP>) -> Option<Box<C>> {
        self.length.fetch_sub(1, Ordering::Relaxed);
        loop {
            gc.pin();
            if self.head.load(Ordering::Acquire).is_null() {
                return None;
            }
            let head = self.head.load(Ordering::Acquire);
            let dropped = unsafe { (*head).next.load(Ordering::Acquire) };

            if let Ok(head) =
                self.head
                    .compare_exchange(head, dropped, Ordering::Release, Ordering::Relaxed)
            {
                let mut data = unsafe { Box::from_raw((*head).data) };
                gc.collect_garbage(head);
                return Some(data);
            }
            gc.unpin();
        }
    }
    /// Returns the length of this [`AtomicStack<C>`].
    pub fn len(&self) -> usize {
        self.length.load(Ordering::Acquire)
    }
    /// Returns true if the [`AtomicStack<C>`] is empty
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
        let gc = &mut self.get_gc();
        while AtomicStack::pop(&self, gc).is_some() {}
    }
}

#[derive(Debug)]
pub struct Node<C> {
    data: *mut C,
    next: AtomicPtr<Node<C>>,
}

impl<C> Node<C> {
    fn new(data: C) -> Box<Self> {
        let ptr = Box::into_raw(Box::new(data));
        Box::new(Node {
            data: ptr,
            next: AtomicPtr::default(),
        })
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

    // #[test]
    // fn stack_unchecked_pop() {
    //     let stack = AtomicStack::new();
    //     let gc = &mut stack.get_gc();
    //     stack.push(1001_usize, gc);
    //     stack.push(1002_usize, gc);
    //     stack.push(1003_usize, gc);
    //     stack.push(1004_usize, gc);
    //     unsafe {
    //         assert_eq!(*stack.unchecked_pop(gc), 1004_usize);
    //         assert_eq!(*stack.unchecked_pop(gc), 1003_usize);
    //         assert_eq!(*stack.unchecked_pop(gc), 1002_usize);
    //         assert_eq!(*stack.unchecked_pop(gc), 1001_usize);
    //     }
    // }

    #[test]
    fn stack_pop() {
        let stack = AtomicStack::new();
        let gc = &mut stack.get_gc();
        stack.push(1002_usize, gc);
        stack.push(1003_usize, gc);
        stack.push(1004_usize, gc);
        assert_eq!(*stack.pop(gc).unwrap(), 1004_usize);
        assert_eq!(*stack.pop(gc).unwrap(), 1003_usize);
        assert_eq!(*stack.pop(gc).unwrap(), 1002_usize);
        assert!(stack.pop(gc).as_ref().is_none());
    }

    #[test]
    fn stack_multi_threading() {
        let stack = AtomicStack::new();
        thread::scope(|s| {
            for _ in 0..10 {
                s.spawn(|| {
                    let gc = &mut stack.get_gc();
                    for _ in 0..500 {
                        stack.push(1008_usize, gc);
                        assert_eq!(1008_usize, *stack.pop(gc).unwrap())
                    }
                });
            }
        });
    }
}
