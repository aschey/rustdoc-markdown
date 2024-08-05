use std::fmt::{Debug, Display};

pub fn plain() {}

pub const fn const_fn() {}

pub fn one_arg(x: usize) {
    println!("{}", x);
}

pub fn return_slice<'a>(input: &'a [usize]) -> &'a [usize] {
    &input
}

pub fn return_raw_pointer(input: &usize) -> *const usize {
    input
}

pub fn return_mut_raw_pointer(input: &mut usize) -> *mut usize {
    input
}

pub fn return_array() -> [u8; 2] {
    [99, 98]
}

pub fn return_iterator() -> impl Iterator<Item = u32> {
    vec![1, 2, 3].into_iter()
}

pub fn generic_arg<T>(t: T) -> T {
    t
}

pub fn generic_bound<T: Sized>(t: T) -> T {
    t
}

pub fn inferred_lifetime(foo: &'_ usize) -> usize {
    *foo
}

pub fn somewhere<T, U>(t: T, u: U)
where
    T: Display,
    U: Debug,
{
    println!("{}, {:?}", t, u);
}

pub fn multiple_bounds<T>(t: T)
where
    T: Debug + Display,
{
}

pub fn multiple_bounds_inline<T: Debug + Display>(t: T) {}

pub fn dyn_arg_one_trait(d: &dyn std::io::Write) {}

pub fn dyn_arg_one_trait_one_lifetime(d: &(dyn std::io::Write + 'static)) {}

pub fn dyn_arg_two_traits(d: &(dyn std::io::Write + Send)) {}

pub fn dyn_arg_two_traits_one_lifetime(d: &(dyn std::io::Write + Send + 'static)) {}

pub unsafe fn unsafe_fn() {}

pub async fn async_fn() {}

pub async fn async_fn_ret_bool() -> bool {
    true
}
