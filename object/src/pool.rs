use crate::{
    object::Object,
    stack::{GarbageCollector, TreiberStack},
};

pub struct ObjectPool<C>
where
    C: Object + Unpin,
{
    stack: TreiberStack<C>,
}

impl<C> ObjectPool<C>
where
    C: Object + Unpin,
{
    fn new() -> Self {
        ObjectPool {
            stack: TreiberStack::new(),
        }
    }
}
