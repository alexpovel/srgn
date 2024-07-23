"""
Module for testing various Python grammar elements.
"""

import asyncio
import os as operating_system
from collections import namedtuple
from math import *
from math import acos as soca
from typing import Dict, List

from ..parent import x
from .sibling import y

# Global variable
test_var: int = 10


# Free function
def free_func():
    """A free function for testing."""
    global test_var
    test_var += 1
    print(f"Global test_var is now {test_var}")


# Decorator for functions
def func_decorator(func):
    """Decorator for free function."""

    def wrapper(*args, **kwargs):
        print("Function decorator called")
        return func(*args, **kwargs)

    return wrapper


@func_decorator
def decorated_func():
    """Function with a decorator."""
    print("Inside decorated function")


# Class definition
class TestClass:
    """Class for testing various features."""

    class_var = "Class variable"

    # Decorator for methods
    @staticmethod
    def static_decorator(func):
        """Decorator for static methods."""

        def wrapper(*args, **kwargs):
            print("Static method decorator called")
            return func(*args, **kwargs)

        return wrapper

    # Class method
    @classmethod
    def class_method(cls) -> None:
        """Class method."""
        cls.class_var += " updated"
        print(f"Class variable is now {cls.class_var}")

    # Method
    def instance_method(self) -> None:
        """Instance method."""
        self.instance_var = "Instance variable"
        print(f"Instance variable is {self.instance_var}")

    @staticmethod
    @static_decorator
    def static_method() -> None:
        """Static method."""
        print("Inside static method")


# Lambda expression
square = lambda x: x * x

# Multiline string
multi_line_str = """
This is a
multi-line string
for testing purposes.
"""

multiline_f_string = f"""This is a
multiline{f_string} string
spanning several lines
"""

raw_string = r"This is a raw string with no special treatment for \n"
bytes_string = b"This is a bytes string"
bytes_string = rf"This is a raw f-string with {raw_string}"


# List comprehension
squared_numbers = ["x" + square(x) for x in range(10)]

# Set comprehension
unique_squares = {square(x) for x in range(10)}

# Dictionary comprehension
squares_dict = {x: square(x) for x in range(10)}


# Exception handling
def exception_handling(x) -> None:
    """Function for testing exceptions."""
    try:
        if x < 0:
            raise ValueError("Negative value")
        elif x == 0:
            raise ZeroDivisionError("Division by zero")
        result = 10 / x
    except ZeroDivisionError as e:
        print(f"Caught an exception: {e}")
    except ValueError as e:
        print(f"Caught an exception: {e}")
    else:
        print("No exceptions caught")
    finally:
        print("This will always be printed")


# Statements
def modify_nonlocal():
    """Function demonstrating nonlocal statement."""
    nonlocal_var = "Initial value"

    def inner():
        nonlocal nonlocal_var
        nonlocal_var = "Modified value"

    inner()
    print(f"Nonlocal variable is {nonlocal_var}")


def inplace_operations():
    """Function demonstrating inplace operators."""
    x = 10
    x += 5
    x -= 3
    x *= 2
    x /= 4
    print(f"Inplace operations result: {x}")


# Control flow
def control_flow():
    """Function demonstrating various control flow statements."""
    # if statement
    if test_var > 5:
        print("test_var is greater than 5")
    else:
        print("test_var is 5 or less")

    # while statement
    counter = 0
    while counter < 3:
        print(f"Counter is {counter}")
        counter += 1

    # for statement
    for i in range(3):
        print(f"Loop iteration {i}")

    # with statement
    with open(__file__) as f:
        content = f.readline()
        print("Read from file:", content)


# Pattern matching
def match_statement(x):
    """Function demonstrating match statement."""
    match x:
        case 0:
            print("Zero")
        case 1:
            print("One")
        case _:
            print("Other")


# Async syntax
async def async_function():
    """Function demonstrating async syntax."""
    await asyncio.sleep(1)
    print("Async function executed")


# Main execution
if __name__ == "__main__":
    free_func()
    decorated_func()
    TestClass.class_method()
    instance = TestClass()
    instance.instance_method()
    TestClass.static_method()
    print(square(5))
    exception_handling(0)
    modify_nonlocal()
    inplace_operations()
    control_flow()
    match_statement(1)
    asyncio.run(async_function())
