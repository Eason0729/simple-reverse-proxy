use std::fmt::Debug;
use std::mem;
use std::sync::atomic::AtomicPtr;
use std::sync::atomic::{AtomicUsize, Ordering};

#[derive(Debug)]
pub struct AtomicStack<C> {
    size: AtomicUsize,
    head: Node<C>,
}

impl<'a, C> AtomicStack<C>
where
    C: Default + Debug + 'a,
{
    fn new() -> Self {
        let head = Node::new(C::default());
        AtomicStack {
            size: AtomicUsize::new(0),
            head,
        }
    }
    fn len(&mut self) -> usize {
        self.size.load(Ordering::Relaxed)
    }
    fn is_empty(&mut self) -> bool {
        self.len() == 0
    }
    fn push(&mut self, data: C) {
        let node = Box::pin(Node::new(data));

        // let node = Box::new(Node::new(data));
        // let node = Box::leak(node);

        let mut head_ptr = self.head.next.load(Ordering::Relaxed);
        let node_ptr = node.next.load(Ordering::Relaxed);

        while self
            .head
            .next
            .compare_exchange(head_ptr, node_ptr, Ordering::Release, Ordering::Relaxed)
            .is_err()
        {
            head_ptr = self.head.next.load(Ordering::Relaxed);
        }
        self.size.fetch_add(1, Ordering::Relaxed);

        mem::forget(node);
        // mem::ManuallyDrop(node);
    }

    fn pop(&mut self) -> C {
        let mut head_ptr = self.head.next.load(Ordering::Relaxed);

        let mut drop_ptr;
        let data;

        unsafe {
            drop_ptr = (*head_ptr).next.load(Ordering::Relaxed);
            data = Box::from_raw(drop_ptr);
        }

        while self
            .head
            .next
            .compare_exchange(head_ptr, drop_ptr, Ordering::Release, Ordering::Relaxed)
            .is_err()
        {
            head_ptr = self.head.next.load(Ordering::Relaxed);
            unsafe {
                drop_ptr = (*head_ptr).next.load(Ordering::Relaxed);
            }
        }

        self.size.fetch_sub(1, Ordering::Relaxed);

        data.data
    }
}

#[derive(Debug)]
struct Node<C> {
    pub data: C,
    next: AtomicPtr<Node<C>>, // Another Node on the heap
}

impl<C> Node<C> {
    /// Creates a new self-referenced [`Node<C>`].
    fn new(data: C) -> Self {
        let mut node = Node {
            data,
            next: AtomicPtr::default(),
        };
        node.next = AtomicPtr::new(&mut node as *mut _);
        node
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
    fn stack_test_p1() {
        let mut stack = AtomicStack::new();
        stack.push(0_usize);
        // verify the next AtomicPtr exist
        dbg!(stack.head.next);
    }

    #[test]
    fn stack_test() {
        let mut stack = AtomicStack::new();
        stack.push(0_usize);
        // try to access the next pointer on the "head"
        let result = stack.pop();
        assert_eq!(result, 0_usize);
    }
}
// cargo test --package object --lib -- pool::test::stack_test --exact --nocapture
