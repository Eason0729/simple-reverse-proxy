use crate::{
    object::Object,
    stack::{AtomicStack, GC},
};

pub struct ObjectPool<C>
where
    C: Object + Unpin,
{
    stack: AtomicStack<C>,
}

impl<C> ObjectPool<C>
where
    C: Object + Unpin,
{
    fn new() -> Self {
        ObjectPool {
            stack: AtomicStack::new(),
        }
    }
}
