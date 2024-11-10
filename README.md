# argcall

argcall is a Rust crate that allows you to make enums and structs callable by associating custom functions with their variants or fields using attribute macros.
This crate provides an elegant way to define callable data structures, enabling each variant or field to execute a unique function and return customizable outputs.

## Features

- Callable Enums and Structs: Define enums and structs that can directly invoke functions upon calling.
- Custom Function Binding: Use attribute macros to specify function names, paths, and arguments for each variant or field.
- Flexible Output Types: Customize return types per variant or struct field to adapt to different use cases.

# Example Usage

```rust
use argcall::Callable;
#[derive(Callable)]
#[argcall(output = i32)]
enum MyEnum {
    #[argcall(fn = one())]
    Unit,
    #[argcall(fn_path = "one")]
    UnitPath,
    UnNamed(Two),
    #[argcall(fn = add(x))]
    Named {
        x: i32
    },
    #[argcall(fn_path = "add")]
    NamedPath{
        x: i32
    }
}

fn one() -> i32 {
    1
}

fn add(x: &i32) -> i32 {
    1 + x
}

struct Two;

impl Callable for Two {
    type Output = i32;

    fn call_fn(&self, _: ()) -> Self::Output {
       2
    }
}

fn main() {
    assert_eq!(MyEnum::Unit.call_fn(()), 1);
    assert_eq!(MyEnum::UnitPath.call_fn(()), 1);
    assert_eq!(MyEnum::UnNamed(Two{}).call_fn(()), 2);
    assert_eq!(MyEnum::Named{x: 2}.call_fn(()), 3);
    assert_eq!(MyEnum::NamedPath{x: 1}.call_fn(()), 2)
}
```

## Explanation

- `#[derive(Callable)]`: Makes MyEnum callable, enabling each variant to be associated with a specific function.
- `#[argcall(fn = <function>)]` and `#[argcall(fn_path = "<function_path>")]`: Specify the function to be called for each variant.
- Each variant calls its associated function when invoked, allowing for custom behavior per variant.

