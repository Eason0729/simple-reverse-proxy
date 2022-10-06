use std::cell::Cell;
use std::f32::consts::E;
use std::ptr::null_mut;
use std::sync::atomic::{AtomicUsize, Ordering};

//  empty  |  inuse  |  active  |
//    finished    inuse     end

/// A non-growable VecDequeue designed for concurrency programing
/// 
/// [`AtomicBuffer<T, CAP>`] allow multiple reader reading or multiple writer writing simultaneously,
/// but no read and write simultaneously
/// 
/// CAP is the max element size of [`AtomicBuffer<T, CAP>`]
#[derive(Debug)]
pub struct AtomicBuffer<T, const CAP: usize> {
    finish: AtomicUsize,
    end: AtomicUsize,
    container: [Cell<*mut T>; CAP],
}

unsafe impl<T, const CAP: usize> Sync for AtomicBuffer<T, CAP> {}

impl<T, const CAP: usize> AtomicBuffer<T, CAP> {
    pub fn new() -> Self {
        let mut container = vec![];
        for _ in 0..CAP {
            container.push(Cell::new(null_mut()));
        }

        AtomicBuffer {
            finish: AtomicUsize::new(0),
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
        self.container[position % CAP].set(data);
    }
    pub fn pop(&self) -> Option<Box<T>> {
        let position = self.finish.fetch_add(1, Ordering::Relaxed);
        return if self.container[position % CAP].get().is_null() {
            self.finish.fetch_sub(1, Ordering::Relaxed);
            None
        } else {
            let ptr = self.container[position % CAP].get();
            self.container[position % CAP].set(null_mut());
            Some(unsafe { Box::from_raw(ptr) })
        };
    }
    pub fn pop_all<F>(&self, f: F)
    where
        F: Fn(Box<T>) + Send + Sync,
    {
        let right_bound = self.end.load(Ordering::Relaxed);
        let left_bound = self.finish.swap(right_bound, Ordering::Relaxed);
        for index in left_bound..right_bound {
            let index = index % CAP;
            f(unsafe { Box::from_raw(self.container[index].get()) });
            self.container[index].set(null_mut());
        }
    }
}

impl<T, const CAP: usize> Drop for AtomicBuffer<T, CAP> {
    fn drop(&mut self) {
        self.pop_all(|b| {
            drop(b);
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
        buffer.push(1002_usize);
        buffer.push(1003_usize);
        assert_eq!(*buffer.pop().unwrap(),1001_usize);
        assert_eq!(*buffer.pop().unwrap(),1002_usize);
        assert_eq!(*buffer.pop().unwrap(),1003_usize);
    }

    #[test]
    fn vec_deque_multi_threading() {
        let vec_dequeue: AtomicBuffer<usize, 1000> = AtomicBuffer::new();
        thread::scope(|s| {
            for _ in 0..10 {
                s.spawn(|| {
                    for _ in 0..100 {
                        vec_dequeue.push(1008_usize);
                    }
                });
            }
        });
        vec_dequeue.pop_all(|d| {
            assert_eq!(*d, 1008_usize);
        });
    }
}
