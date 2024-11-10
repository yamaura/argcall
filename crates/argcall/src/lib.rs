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

pub trait Tuple {}
impl Tuple for () {}

pub trait Callable<Args: Tuple = ()> {
    type Output;
    fn call_fn(&self, args: Args) -> Self::Output;
}
