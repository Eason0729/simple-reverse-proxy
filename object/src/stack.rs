use std::fmt::Debug;
use std::mem;
use std::sync::atomic::AtomicPtr;
use std::sync::atomic::{AtomicUsize, Ordering};

#[derive(Debug)]
pub struct AtomicStack<C> {
    size: AtomicUsize,
    head: Node<C>,
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
    fn as_mut_ref(&mut self) -> &mut C {
        let node;
        unsafe {
            node = &mut *self.content;
        }
        &mut node.data
    }
    fn as_ref(&self) -> &C {
        let node;
        unsafe {
            node = &*self.content;
        }
        &node.data
    }
}

impl<'a, C> AtomicStack<C>
where
    C: Default + Debug + 'a,
{
    pub fn new() -> Self {
        let head = *Node::new(C::default());
        AtomicStack {
            size: AtomicUsize::new(0),
            head,
        }
    }
    pub fn len(&mut self) -> usize {
        self.size.load(Ordering::Relaxed)
    }
    pub fn is_empty(&mut self) -> bool {
        self.len() == 0
    }
    pub fn push(&mut self, data: C) {
        let node = Node::leak_new(data);

        unsafe {
            let node = &mut *node;
            while !node.swap_next(&self.head) {}
        }
        self.size.fetch_add(1, Ordering::Relaxed);
    }

    pub fn pop(&mut self) -> Option<Wrapper<C>> {
        if self.sub_or_empty() {
            None
        } else {
            Some(self.unchecked_pop())
        }
    }

    pub unsafe fn unchecked_pop(&mut self) -> Wrapper<C> {
        let head = &self.head;
        let next = head.next.load(Ordering::Relaxed);
        unsafe {
            while !head.swap_next(&*next) {
                let next = head.next.load(Ordering::Relaxed);
            }
        }
        let dropped = next;
        Wrapper { content: dropped }
    }

    #[cfg(test)]
    fn peek(&mut self) -> &C {
        let node;
        unsafe {
            node = &*self.head.next.load(Ordering::Relaxed);
        }
        &node.data
    }
    fn sub_or_empty(&mut self) -> bool {
        let size = self.size.get_mut();
        return if size == &0_usize {
            true
        } else {
            *size = 0_usize;
            false
        };
    }
}

#[derive(Debug)]
struct Node<C> {
    pub data: C,
    next: AtomicPtr<Node<C>>,
}

impl<C> Node<C> {
    /// Creates a new self-referenced [`Node<C>`].
    fn new(data: C) -> Box<Self> {
        let node = Node {
            data,
            next: AtomicPtr::default(),
        };
        let mut node = Box::new(node);
        node.next = AtomicPtr::new(&mut *node as *mut _);
        node
    }
    fn leak_new(data: C) -> *mut Self {
        let node = Self::new(data);
        let ptr = Box::leak(node);
        ptr
    }
    fn swap_next(&self, node: &Node<C>) -> bool {
        let self_next = self.next.load(Ordering::Relaxed);
        let node_next = node.next.load(Ordering::Relaxed);

        self.next
            .compare_exchange(self_next, node_next, Ordering::Release, Ordering::Relaxed)
            .is_ok()
    }
}

// pub struct LinkRing<T> {
//     pub data: T,
//     next: NonNull<AtomicPtr<LinkRing<T>>>,
// }

// impl<T> LinkRing<T>
// where
//     T: Clone,
// {
//     fn new(inner: Vec<T>) -> LinkRing<T> {
//         #[cfg(debug_assertions)]
//         assert!(inner.len() > 0);

//         let mut previous: Option<LinkRing<T>> = None;

//         for iter in 0..inner.len() {
//             let mut current = LinkRing {
//                 data: inner[iter].clone(),
//                 next: NonNull::<AtomicPtr<LinkRing<T>>>::dangling(),
//             };
//             let mut current_ptr = AtomicPtr::new(&mut current as *mut LinkRing<T>);

//             current.next = NonNull::new(&mut current_ptr as *mut _).unwrap();

//             if iter != 0 {
//                 swap(&mut current.next, &mut previous.unwrap().next);
//             }

//             previous = Some(current);
//         }
//         previous.unwrap()
//     }
// }

// pub struct Pool<T>
// where
//     T: Object,
// {
//     datas: LinkRing<T>,
// }

// pub struct Container<'a, T>
// where
//     T: Object,
// {
//     data: T,
//     pool: &'a Pool<T>,
// }

#[cfg(test)]
mod test {
    use std::sync::atomic::Ordering;

    use super::*;

    #[test]
    fn stack_test() {
        let mut stack = AtomicStack::new();
        stack.push(0_usize);
        let result = stack.peek();
        assert_eq!(*result, 0_usize);
    }

    #[test]
    fn stack_pop() {
        let mut stack = AtomicStack::new();
        stack.push(0_usize);
        let result = stack.pop().unwrap();
        assert_eq!(*result.as_ref(), 0_usize);
    }
}
