use std::marker::PhantomData;
use std::sync::atomic::Ordering;
use std::{ptr::null_mut, sync::atomic::AtomicUsize};

use super::buffer::AtomicBuffer;

struct Bag<T>{
    data:Vec<*mut T>
}

impl<T> Bag<T> {
    fn new()->Self{
        Bag{data:vec![]}
    }
    fn push(&mut self,data:*mut T){
        self.data.push(data);
    }
    fn empty(&mut self)->Bag<T>{
        let bag=Bag{data:self.data.clone()};
        self.data=vec![];
        bag
    }
}

impl<T> Drop for Bag<T>{
    fn drop(&mut self) {
        for ptr in &self.data{
            // dangerous
            drop(unsafe{Box::from_raw(ptr.clone())});
        }
    }
}

struct Global<T, const CAP: usize> {
    epoch: AtomicUsize,
    bags: [AtomicBuffer<Bag<T>,CAP>;3],
    status: [AtomicUsize; 3],
}

impl<T, const CAP: usize> Global<T, CAP> {
    fn new() -> Global<T, CAP> {
        Global {
            epoch: AtomicUsize::new(0),
            bags: [AtomicBuffer::new(),AtomicBuffer::new(),AtomicBuffer::new(),],
            status: [
                AtomicUsize::new(0),
                AtomicUsize::new(0),
                AtomicUsize::new(0),
            ],
        }
    }
    fn get_epoch(&self)->usize{
        // remember to consider the affect of other thread
        let epoch=self.epoch.load(Ordering::Relaxed);
        // when synchronizing, check if global fullfill the condition to advance to next epoch
        if self.status[(epoch+3-1)%3].load(Ordering::Relaxed)==0{
            // recycle the previous epoch(bags)
            let previous_epoch=(epoch+3-1)%3;
            // warning: consider pin when cleaning up
            // warning: datarace
            self.status[previous_epoch].fetch_add(1, Ordering::Acquire);
            self.bags[previous_epoch].pop_iter(|_|{
            });
            self.epoch.compare_exchange(epoch, epoch+1,Ordering::AcqRel,Ordering::Acquire).is_ok();
            self.status[previous_epoch].fetch_sub(1, Ordering::Acquire);
        }
        epoch
    }
}

struct Local<'a, T, const LIMIT: usize, const CAP: usize> {
    epoch: usize,
    size: usize,
    buffer: Bag<T>,
    pinned: usize,
    global: &'a Global<T, CAP>,
}

impl<'a, T, const LIMIT: usize, const CAP: usize> Local<'a, T, LIMIT, CAP> {
    fn new(global: &'a Global<T, CAP>) -> Local<'a, T, LIMIT, CAP> {
        Local {
            epoch: 0,
            size: 0,
            pinned: 0,
            buffer: Bag::new(),
            global,
        }
    }
    pub fn pin(&mut self) {
        self.pinned += 1;
        if self.pinned == 1 {
            self.global.status[self.epoch].fetch_add(1, Ordering::Acquire);
        }
    }
    pub fn unpin(&mut self) {
        self.pinned -= 1;
        if self.pinned == 0 {
            self.global.status[self.epoch].fetch_sub(1, Ordering::Acquire);
        }
    }
    pub fn collect_garbage(&mut self, garbage: *mut T) {
        // check local buffer, mirgiate garbage to global if full
        if self.size == LIMIT {
            // warning: should I use local epoch?
            self.global.bags[self.epoch].push(self.buffer.empty());
        }
        // check local pin, synchronize to global epoch if unpinned
        if self.pinned==0{
            self.epoch=self.global.get_epoch();
        }
        self.buffer.push(garbage);

        self.size += 1;
    }
}