use std::fmt::Debug;
use std::mem::MaybeUninit;
use std::pin::Pin;
use std::sync::atomic::{AtomicPtr, AtomicUsize, Ordering};

#[derive(Debug)]
pub struct TreiberStack<C>
where
    C: Unpin,
{
    head: AtomicPtr<Node<C>>,
}

impl<C> TreiberStack<C>
where
    C: Unpin,
{
    pub fn new() -> Self {
        let node = Box::into_raw(Node::new(MaybeUninit::uninit()));
        let head = AtomicPtr::new(node);
        unsafe {
            (&*head.load(Ordering::SeqCst))
                .next
                .store(node, Ordering::SeqCst);
        }
        TreiberStack { head }
    }
    pub fn push(&self, data: C) {
        let node = Node::new(MaybeUninit::new(data));
        let node = Box::leak(node);

        loop {
            let mut head = self.head.load(Ordering::Relaxed);
            node.next.store(head, Ordering::Relaxed);

            if self
                .head
                .compare_exchange(head, node, Ordering::Release, Ordering::Relaxed)
                .is_ok()
            {
                break;
            }
        }
    }
    pub unsafe fn unchecked_pop(&self) -> GarbageCollector<C> {
        let mut head;
        loop {
            head = self.head.load(Ordering::Acquire);
            let dropped = unsafe { (*head).next.load(Ordering::Acquire) };
            if self
                .head
                .compare_exchange(head, dropped, Ordering::Release, Ordering::Relaxed)
                .is_ok()
            {
                break;
            }
        }

        let data = Box::from_raw(head);
        GarbageCollector { node: data }
    }

    pub fn pop(&self) -> Option<GarbageCollector<C>> {
        // match self
        //     .head
        //     .fetch_update(Ordering::Release, Ordering::Relaxed, |head| {
        //         let dropped = unsafe { (*head).next.load(Ordering::Acquire) };
        //         if head == dropped {
        //             None
        //         } else {
        //             Some(dropped)
        //         }
        //     }) {
        //     Ok(head) => {
        //         let data = unsafe { Box::from_raw(head) };
        //         Some(GarbageCollector { node: data })
        //     }
        //     Err(_) => None,
        // }
        let mut head;
        loop {
            head = self.head.load(Ordering::Acquire);
            let dropped = unsafe { (*head).next.load(Ordering::Acquire) };
            if head == dropped {
                return None;
            }
            if self
                .head
                .compare_exchange(head, dropped, Ordering::Release, Ordering::Relaxed)
                .is_ok()
            {
                break;
            }
        }
        let data = unsafe { Box::from_raw(head) };
        Some(GarbageCollector { node: data })
    }
}

impl<C> Drop for TreiberStack<C>
where
    C: Unpin,
{
    fn drop(&mut self)
    where
        C: Unpin,
    {
        while TreiberStack::pop(&self).is_some() {}
        unsafe {
            drop(Box::from_raw(self.head.load(Ordering::Acquire)));
        }
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

pub struct GarbageCollector<T> {
    node: Box<Node<T>>,
}

impl<C> AsMut<C> for GarbageCollector<C> {
    fn as_mut(&mut self) -> &mut C {
        unsafe { self.node.data.assume_init_mut() }
    }
}

impl<C> AsRef<C> for GarbageCollector<C> {
    fn as_ref(&self) -> &C {
        unsafe { self.node.data.assume_init_ref() }
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
    fn stack_unchecked_pop() {
        let stack = TreiberStack::new();
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
        let stack = TreiberStack::new();
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
        let stack = TreiberStack::new();
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
