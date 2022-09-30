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
