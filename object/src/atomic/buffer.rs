use std::sync::atomic::AtomicUsize;
use std::sync::atomic::{AtomicPtr, Ordering};

//  empty  |  inuse  |  active  |
//    finished    inuse     end
#[derive(Debug)]
pub struct AtomicBuffer<T, const CAP: usize> {
    finish: AtomicUsize,
    // inuse: AtomicUsize,
    end: AtomicUsize,
    container: [AtomicPtr<T>; CAP],
}

impl<T, const CAP: usize> AtomicBuffer<T, CAP> {
    pub fn new() -> Self {
        let mut container = vec![];
        for _ in 0..CAP {
            container.push(AtomicPtr::default());
        }

        AtomicBuffer {
            finish: AtomicUsize::new(0),
            // inuse: AtomicUsize::new(0),
            end: AtomicUsize::new(0),
            container: container.try_into().unwrap(),
        }
    }
    pub fn push(&self, data: T) {
        let data = Box::into_raw(Box::new(data));
        self.push_ptr(data);
    }
    fn push_ptr(&self, data: *mut T) {
        let position = self.end.fetch_add(1, Ordering::Relaxed);
        self.container[position % CAP].store(data, Ordering::Relaxed);
        // self.inuse.fetch_add(1, Ordering::Relaxed);
    }
    pub fn pop_iter<F>(&self, f: F)
    where
        F: Fn(&mut T) + Send + Sync,
    {
        self.pop_iter_ptr(|ptr| unsafe {
            let reference = &mut *ptr;
            f(reference);
            drop(Box::from_raw(ptr));
        });
    }
    pub fn pop_iter_ptr<F>(&self, f: F)
    where
        F: Fn(*mut T) + Send + Sync,
    {
        let right_bound = self.end.load(Ordering::Relaxed);
        let left_bound = self.finish.swap(right_bound, Ordering::Relaxed);
        for index in left_bound..right_bound {
            let index = index % CAP;
            f(self.container[index].load(Ordering::Relaxed));
        }
    }
}

impl<T, const CAP: usize> Drop for AtomicBuffer<T, CAP> {
    fn drop(&mut self) {
        self.pop_iter_ptr(|ptr| {
            drop(unsafe { Box::from_raw(ptr) });
        });
    }
}

#[cfg(test)]
mod test {
    use std::thread;

    use super::AtomicBuffer;

    #[test]
    fn vec_deque_push() {
        let buffer: AtomicBuffer<usize, 128> = AtomicBuffer::new();
        buffer.push(1001_usize);
    }

    #[test]
    fn vec_deque() {
        let buffer: AtomicBuffer<usize, 128> = AtomicBuffer::new();
        buffer.push(1001_usize);
        buffer.push(1001_usize);
        buffer.push(1001_usize);
        buffer.pop_iter(|x| assert_eq!(*x, 1001_usize));
    }

    // #[test]
    // fn vec_deque_multi_threading() {
    //     let vec_dequeue: AtomicBuffer<usize, 1024> = AtomicBuffer::new();
    //     let vec_dequeue = Box::leak(Box::new(vec_dequeue));
    //     let mut threads = vec![];
    //     for _ in 0..10 {
    //         threads.push(thread::spawn(|| {
    //             for _ in 0..100 {
    //                 vec_dequeue.push(1008_usize);
    //             }
    //         }));
    //     }
    //     for _ in 0..5 {
    //         threads.push(thread::spawn(|| {
    //             for _ in 0..100 {
    //                 vec_dequeue.pop_iter(|d| {
    //                     assert_eq!(unsafe { *d }, 1008_usize);
    //                 });
    //             }
    //         }));
    //     }
    // }
}
