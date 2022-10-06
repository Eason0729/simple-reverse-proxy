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

macro_rules! impl_reuse {
    ($i:ident) => {
        impl Object for $i {
            fn reuse(self: &mut Self) {}
        }
    };
}

impl_reuse!(i8);
impl_reuse!(i16);
impl_reuse!(i64);
impl_reuse!(i128);
impl_reuse!(f32);
impl_reuse!(f64);
impl_reuse!(u8);
impl_reuse!(u16);
impl_reuse!(u64);
impl_reuse!(u128);
