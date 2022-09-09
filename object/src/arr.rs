use std::ops::Index;

pub trait Object
where
    Self: Sized,
{
    fn reuse(self: &mut Self) {}
}

impl<T> Object for Vec<T> {
    fn reuse(self: &mut Self) {
        self.clear()
    }
}

impl<T, const C: usize> Object for [T; C]
where
    T: Sized,
{
    fn reuse(self: &mut Self) {}
}

macro_rules! empty_reuse {
    ($i:ident) => {
        impl Object for $i {
            fn reuse(self: &mut Self) {}
        }
    };
}

empty_reuse!(i8);
empty_reuse!(i16);
empty_reuse!(i64);
empty_reuse!(i128);
empty_reuse!(f32);
empty_reuse!(f64);
empty_reuse!(u8);
empty_reuse!(u16);
empty_reuse!(u64);
empty_reuse!(u128);
