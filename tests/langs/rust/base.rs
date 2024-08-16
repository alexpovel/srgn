//! Module for testing various Rust grammar elements.

use a::b::{c, d, e::f, g::h::i};
use a::item as b_item;
use something::prelude::*;
use std::collections::HashMap;
use std::collections::HashSet;
use std::fs::File;
use std::io::{self, BufRead, BufReader, Write};
use std::sync::atomic::{AtomicI32, Ordering};

#[macro_use]
extern crate lazy_static;

mod parent {
    pub fn x() {
        println!("Function x from parent module");
    }
}

mod sibling {
    pub fn y() {
        println!("Function y from sibling module");
    }
}

// Global variable
static TEST_VAR: AtomicI32 = AtomicI32::new(10);

/// Free function
///
/// With a docstring.
fn free_func() {
    // A free function for testing.
    let mut test_var = TEST_VAR.load(Ordering::SeqCst);
    test_var += 1;
    TEST_VAR.store(test_var, Ordering::SeqCst);
    println!("Global test_var is now {}", test_var);
}

// Decorator for functions
fn func_decorator<F>(func: F)
where
    F: Fn() + 'static,
{
    // Decorator for free function.
    fn wrapper<F>(func: F)
    where
        F: Fn(),
    {
        println!("Function decorator called");
        func();
    }

    wrapper(func);
}

fn decorated_func() {
    // Function with a decorator.
    println!("Inside decorated function");
}

/// Struct definition
///
/// Also has a docstring. With code:
///
/// ```
/// let x = 3;
/// ```
struct TestStruct {
    instance_var: String,
}

impl TestStruct {
    fn new() -> Self {
        TestStruct {
            instance_var: String::from("hello"),
        }
    }

    /*
        More comment types
    */

    // Static decorator for methods
    fn static_decorator<F>(func: F) -> impl Fn()
    where
        F: Fn(),
    {
        // Decorator for static methods.
        move || {
            println!("Static method decorator called");
            func();
        }
    }

    // Method
    fn instance_method(&mut self) {
        // Instance method.
        self.instance_var = "Instance variable".to_string();
        println!("Instance variable is {}", self.instance_var);
    }

    fn static_method() {
        // Static method.
        println!("Inside static method");
    }
}

// Enum definition
enum TestEnum {
    VariantOne,
    VariantTwo(String),
    VariantThree { a: String, b: u64 },
}

impl TestEnum {
    fn match_statement(x: TestEnum) {
        // Function demonstrating match statement.
        match x {
            TestEnum::VariantOne => println!("One"),
            TestEnum::VariantTwo => println!("Two"),
            TestEnum::VariantOther => println!("Other"),
        }
    }
}

// Statements
fn modify_nonlocal() {
    let mut nonlocal_var = "Initial value".to_string();

    {
        let mut inner = || {
            nonlocal_var = "Modified value".to_string();
        };
        inner();
    }
    println!("Nonlocal variable is {}", nonlocal_var);
}

fn inplace_operations() {
    // Function demonstrating inplace operators.
    let mut x = 10;
    x += 5;
    x -= 3;
    x *= 2;
    x /= 4;
    println!("Inplace operations result: {}", x);
}

// Control flow
fn control_flow() {
    // Function demonstrating various control flow statements.
    // if statement
    if TEST_VAR.load(Ordering::SeqCst) > 5 {
        println!("test_var is greater than 5");
    } else {
        println!("test_var is 5 or less");
    }

    // while statement
    let mut counter = 0;
    while counter < 3 {
        println!("Counter is {}", counter);
        counter += 1;
    }

    // for statement
    for i in 0..3 {
        println!("Loop iteration {}", i);
    }

    // with statement
    let file = File::open(file!()).expect("Cannot open file");
    let reader = BufReader::new(file);
    if let Some(line) = reader.lines().next() {
        println!("Read from file: {:?}", line);
    }
}

#[tokio::main]
async fn async_main() -> Result<(), ()> {
    // Open a connection to the mini-redis address.
    let mut client = client::connect("127.0.0.1:6379").await?;

    // Set the key "hello" with value "world"
    client.set("hello", "world".into()).await?;

    // Get key "hello"
    let result = client.get("hello").await?;

    println!("got value from the server; result={:?}", result);

    Ok(())
}

pub async fn async_pub_fn() -> Result<(), ()> {
    Ok(())
}

// Main execution
fn main() {
    use std::fs::read_to_string;

    // Lambda expression
    let square = |x: i32| -> i32 { x * x };

    // Multiline string
    let multi_line_str = "
This is a
multi-line string
for testing purposes.
";

    let multiline_f_string = format!(
        "This is a\nmultiline{} string\nspanning several lines",
        "{f_string}"
    );

    let raw_string = r"This is a raw string with no special treatment for \n";
    let raw_multiline_string = r#"
This is a raw string with no special treatment for \n
"#;
    let bytes_string = b"This is a bytes string";
    let raw_f_string = format!(r"This is a raw f-string with {}", raw_string);

    let check = |s: &str| s.ends_with("something");

    free_func();
    func_decorator(decorated_func);
    let mut instance = TestStruct {
        instance_var: String::new(),
    };
    instance.instance_method();
    TestStruct::static_decorator(TestStruct::static_method)();
    println!("{}", square(5));
    modify_nonlocal();
    inplace_operations();
    control_flow();
    TestEnum::match_statement(TestEnum::VariantOne);
}

#[cfg(feature = "some-feature")]
#[allow(clippy::single_match_else)]
#[cfg(doc)]
fn function_with_attributes() {}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn some_test() {
        assert_eq!(3, 3);
    }

    #[rstest]
    #[case(3)]
    fn test_with_rstest(#[case] i: u64) {
        assert_eq!(1337, 1337);
    }
}

pub(crate) fn function_pub_crate() {}

pub(self) fn function_pub_self() {}

pub(super) fn function_pub_super() {}

pub struct PubStruct {}
pub(crate) struct PubCrateStruct {}
pub(self) struct PubSelfStruct {}
pub(super) struct PubSuperStruct {}

pub enum PubEnum {}
pub(crate) enum PubCrateEnum {}
pub(self) enum PubSelfEnum {}
pub(super) enum PubSuperEnum {}

#[repr(C)]
union MyUnion {
    f1: u32,
    f2: f32,
}

const fn const_function() {}
pub const fn const_function_pub() {}
pub(crate) const fn const_function_pub_crate() {}
pub(self) const fn const_function_pub_self() {}
pub(super) const fn const_function_pub_super() {}

unsafe fn perform_unsafe_operation() {
    let ptr = 0x1234 as *mut i32;
    *ptr = 42;
}

extern "C" fn example_extern() -> i32 {
    42
}

impl Y {}
impl<T> Y<T> {}

trait Convertible<T> {
    fn convert(&self) -> T;
}

impl<T, U> Convertible<U> for T
where
    T: AsRef<str>,
    U: From<String>,
{
    fn convert(&self) -> U {
        U::from(self.as_ref().to_owned())
    }
}

impl X for Y {}
impl<T> X for Y<T> {}
impl<T> X<T> for Y {}

impl PubStruct {
    const PANIC: () = panic!("bam!");
}
