use std::fmt::Debug;
use std::mem::MaybeUninit;
use std::sync::atomic::{AtomicPtr, AtomicUsize, Ordering};

#[derive(Debug)]
pub struct AtomicStack<C> {
    size: AtomicUsize,
    head: Box<Node<C>>,
}

pub struct Wrapper<C> {
    content: *mut Node<C>,
}

impl<C> Drop for Wrapper<C> {
    fn drop(&mut self) {
        unsafe {
            let drop_node = Box::from_raw(self.content);
            drop(drop_node);
        }
    }
}

impl<C> Wrapper<C> {
    #[inline]
    fn as_mut_ref(&mut self) -> &mut C {
        let node;
        unsafe {
            node = &mut *self.content;
            node.data.assume_init_mut()
        }
    }
    #[inline]
    fn as_ref(&self) -> &C {
        let node;
        unsafe {
            node = &*self.content;
            node.data.assume_init_ref()
        }
    }
}

impl<'a, C> AtomicStack<C> {
    #[inline]
    pub fn new() -> Self {
        let head = Node::dangling();
        AtomicStack {
            size: AtomicUsize::new(0),
            head,
        }
    }
    #[inline]
    pub fn len(&self) -> usize {
        self.size.load(Ordering::Relaxed)
    }
    #[inline]
    pub fn is_empty(&self) -> bool {
        self.len() == 0
    }
    #[inline]
    pub fn push(&self, data: C) {
        let node = Node::leak_new(data);
        unsafe { while !self.head.swap_next(&*node) {} }
        self.size.fetch_add(1, Ordering::Release);
    }

    #[inline]
    pub fn pop(&self) -> Option<Wrapper<C>> {
        if self.sub_or_empty() {
            None
        } else {
            unsafe { Some(self.unchecked_pop()) }
        }
    }

    #[inline]
    unsafe fn unchecked_pop(&self) -> Wrapper<C> {
        while !self
            .head
            .swap_next(&*self.head.next.load(Ordering::Acquire))
        {}
        let dropped = self.head.next.load(Ordering::Acquire);
        Wrapper { content: dropped }
    }

    #[inline]
    fn sub_or_empty(&self) -> bool {
        let mut result = true;
        while self
            .size
            .fetch_update(Ordering::Acquire, Ordering::Acquire, |size| {
                if size == 0_usize {
                    result = true;
                    Some(size)
                } else {
                    result = false;
                    Some(size - 1)
                }
            })
            .is_err()
        {}
        result
    }
}

impl<C> Drop for AtomicStack<C> {
    fn drop(&mut self) {
        #[cfg(test)]
        println!("dropping AtomicStack with {:?} elements", self.len());
        while let Some(_) = self.pop() {}
    }
}

#[derive(Debug)]
struct Node<C> {
    data: MaybeUninit<C>,
    next: AtomicPtr<Node<C>>,
}

impl<C> Node<C> {
    /// Creates a new self-referenced [`Node<C>`].
    #[inline]
    fn new(data: MaybeUninit<C>) -> Box<Self> {
        let node = Node {
            data: data,
            next: AtomicPtr::default(),
        };
        let mut node = Box::new(node);
        node.next = AtomicPtr::new(&mut *node);
        node
    }
    #[inline]
    fn dangling() -> Box<Self> {
        Self::new(MaybeUninit::uninit())
    }
    #[inline]
    fn leak_new(data: C) -> *mut Self {
        let node = Self::new(MaybeUninit::new(data));
        Box::into_raw(node)
    }
    #[inline]
    fn swap_next(&self, node: &Node<C>) -> bool {
        let self_next = self.next.load(Ordering::Relaxed);
        let node_next = node.next.load(Ordering::Relaxed);

        self.next
            .compare_exchange(self_next, node_next, Ordering::SeqCst, Ordering::SeqCst)
            .is_ok()
    }
}

#[cfg(test)]
impl<C> Drop for Node<C> {
    fn drop(&mut self) {
        unsafe {
            let expect_self = self.next.load(Ordering::Relaxed);
            let next = (&*expect_self).next.load(Ordering::Relaxed);
            assert_eq!(expect_self, next);
            println!("drop node");
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
    fn stack_peek() {
        let stack: AtomicStack<usize> = AtomicStack::new();
        stack.push(0_usize);
        stack.push(1_usize);
        stack.push(2_usize);
        // try to peek stack to test if it leak in miri
        unsafe {
            dbg!(&*stack.head.next.load(Ordering::Relaxed));
        }
    }

    #[test]
    fn stack_pop() {
        let stack = AtomicStack::new();
        stack.push(2_usize);
        stack.push(3_usize);
        assert_eq!(*stack.pop().unwrap().as_ref(), 3_usize);
        assert_eq!(*stack.pop().unwrap().as_ref(), 2_usize);
    }

    #[test]
    fn stack_multi_threading() {
        let stack = AtomicStack::new();
        let stack = Box::leak(Box::new(stack));
        let mut threads = vec![];
        for _ in 0..10 {
            threads.push(thread::spawn(|| {
                for _ in 0..100 {
                    stack.push(3_usize);
                    assert_eq!(3_usize, *stack.pop().unwrap().as_ref())
                }
            }));
        }
    }
}
