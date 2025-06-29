#![doc = include_str!("../README.md")]

/// The `Callable` macro derives functionality that enables enums and structs to be directly callable,
/// associating custom functions or methods with specific variants or fields.
///
/// ## Usage
///
/// The `#[derive(Callable)]` macro can be applied to enums and structs to make them callable by defining
/// a designated output type and associating specific functions with each variant or field. This is particularly
/// useful when each variant or field should execute different logic upon calling.
///
/// ### Attributes
///
/// - `#[argcall(output = <Type>)]`: Specifies the return type for the `call_fn` method. This should match the
///   output type of the functions bound to the variants or fields.
/// - `#[argcall(fn = <function()>)]`: Binds a specific function to the variant. The function is invoked when
///   `call_fn` is called on the variant.
/// - `#[argcall(fn_path = "<function_path>")]`: Binds a function by path, allowing the use of functions
///   located in other modules or namespaces.
/// - `#[argcall(fn = <function(arg)>) or fn_path = "<function_path(arg)>"]`: Allows binding a function with
///   an argument, typically used for named fields that provide a specific value to the function.
pub use argcall_derive::Callable;
pub use argcall_derive::CallableMut;
pub use argcall_derive::CallableOnce;

#[cfg(feature = "async")]
use core::future::{Future, Ready, ready};

pub trait Tuple {}
impl Tuple for () {}

pub trait Callable<Args: Tuple = ()> {
    type Output;
    fn call_fn(&self, args: Args) -> Self::Output;

    #[cfg(feature = "async")]
    fn call_fn_async(&self, args: Args) -> Ready<Self::Output> {
        ready(self.call_fn(args))
    }
}

pub trait CallableMut<Args: Tuple = ()> {
    type Output;
    fn call_fn_mut(&mut self, args: Args) -> Self::Output;

    #[cfg(feature = "async")]
    fn call_fn_async_mut(&mut self, args: Args) -> Ready<Self::Output> {
        ready(self.call_fn_mut(args))
    }
}

pub trait CallableOnce<Args: Tuple = ()> {
    type Output;
    fn call_fn_once(self, args: Args) -> Self::Output;

    #[cfg(feature = "async")]
    fn call_fn_async_once(self, args: Args) -> Ready<Self::Output>
    where
        Self: Sized,
    {
        ready(self.call_fn_once(args))
    }
}

impl<T, Args: Tuple> CallableMut<Args> for T
where
    T: Callable<Args>,
{
    type Output = T::Output;
    fn call_fn_mut(&mut self, args: Args) -> Self::Output {
        self.call_fn(args)
    }
}

impl<T, Args: Tuple> CallableOnce<Args> for T
where
    T: CallableMut<Args>,
{
    type Output = T::Output;
    fn call_fn_once(mut self, args: Args) -> Self::Output {
        self.call_fn_mut(args)
    }
}

#[cfg(feature = "async")]
/// An asynchronous callable trait.
///
/// This trait is analogous to the synchronous `Callable` trait but returns a future.
/// The associated type `Future` is any type implementing `Future`, and when resolved,
/// will yield a value of type `Output`.
pub trait AsyncCallable<Args: Tuple = ()> {
    /// The output type produced by the asynchronous call.
    type Output;
    /// The future type that will eventually resolve to `Self::Output`.
    type Future: Future<Output = Self::Output>;

    /// Asynchronously calls the bound function for the instance with the specified arguments,
    /// returning a future that yields the result.
    fn call_fn_async(&self, args: Args) -> Self::Future;
}

#[cfg(feature = "async")]
/// A mutable asynchronous callable trait.
///
/// This is analogous to `CallableMut` and allows the method to be called on a mutable reference.
pub trait AsyncCallableMut<Args: Tuple = ()> {
    type Output;
    type Future: Future<Output = Self::Output>;

    /// Asynchronously calls the bound function using a mutable reference.
    fn call_fn_async_mut(&mut self, args: Args) -> Self::Future;
}

#[cfg(feature = "async")]
/// Provide a default implementation of `AsyncCallableMut` for any type that already implements `AsyncCallable`.
impl<T, Args: Tuple> AsyncCallableMut<Args> for T
where
    T: AsyncCallable<Args>,
{
    type Output = T::Output;
    type Future = T::Future;

    fn call_fn_async_mut(&mut self, args: Args) -> Self::Future {
        // Forward the call to the immutable version.
        self.call_fn_async(args)
    }
}

#[cfg(feature = "async")]
/// An asynchronous callable trait that consumes the instance.
///
/// This is analogous to `CallableOnce` and allows the call function to take ownership.
pub trait AsyncCallableOnce<Args: Tuple = ()> {
    type Output;
    type Future: Future<Output = Self::Output>;

    /// Asynchronously calls the bound function, consuming the instance,
    /// and returns a future that yields the result.
    fn call_fn_async_once(self, args: Args) -> Self::Future;
}

#[cfg(feature = "async")]
/// Provide a default implementation of `AsyncCallableOnce` for any type that already implements `AsyncCallableMut`.
impl<T, Args: Tuple> AsyncCallableOnce<Args> for T
where
    T: AsyncCallableMut<Args>,
{
    type Output = T::Output;
    type Future = T::Future;

    fn call_fn_async_once(mut self, args: Args) -> Self::Future {
        // Forward the call to the mutable version.
        self.call_fn_async_mut(args)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[cfg(feature = "async")]
    use pollster::FutureExt as _;

    fn assert_send<T: Send>() {}
    fn assert_sized<T: Sized>() {}

    #[test]
    fn test_unit_is_send() {
        assert_send::<()>();
    }

    #[test]
    fn test_tuple_unit_is_send() {
        struct Wrapper<T: Tuple>(T);
        assert_send::<Wrapper<()>>();
    }

    #[test]
    fn test_callable() {
        struct MyCallable;

        impl Callable for MyCallable {
            type Output = i32;
            fn call_fn(&self, _: ()) -> Self::Output {
                42
            }
        }

        let callable = MyCallable;
        assert_eq!(callable.call_fn(()), 42);
        assert_sized::<MyCallable>();
    }

    #[cfg(feature = "async")]
    #[test]
    fn test_async_callable() {
        struct MyAsyncCallable;

        impl AsyncCallable for MyAsyncCallable {
            type Output = i32;
            type Future = Ready<Self::Output>;

            fn call_fn_async(&self, _: ()) -> Self::Future {
                ready(42)
            }
        }

        let async_callable = MyAsyncCallable;
        assert_eq!(
            async { async_callable.call_fn_async(()).await }.block_on(),
            42
        );
    }
}
