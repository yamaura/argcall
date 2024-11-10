pub use argcall_derive::Callable;

pub trait Tuple {}
impl Tuple for () {}

pub trait Callable<Args: Tuple = ()> {
    type Output;
    fn call_fn(&self, args: Args) -> Self::Output;
}
