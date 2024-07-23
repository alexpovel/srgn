// Import necessary modules
import * as fs from "fs";
import 'some/package';
import { promisify } from "util";
const sleep = promisify(setTimeout);

// Global variable
let testVar: number = 10;

/**
 * Proper function documentation.
 * @param a Bla
 * @param b Bla.
 * @return Does not exist.
*/
function freeFunc(): void {
    testVar += 1;
    console.log(`Global testVar is now ${testVar}`);
}

// Decorator for functions
function funcDecorator(func: Function): Function {
    return function(...args: any[]): any {
        console.log("Function decorator called");
        return func(...args);
    };
}

// Function with a decorator
const decoratedFunc = funcDecorator(() => {
    console.log("Inside decorated function");
});

// Class definition
class TestClass {
    static classVar: string = "Class variable";

    instanceVar: string;

    static classMethod(): void {
        this.classVar += " updated";
        console.log(`Class variable is now ${this.classVar}`);
    }

    instanceMethod(): void {
        this.instanceVar = 'Instance variable'; // Single quotes
        console.log(`Instance variable is ${this.instanceVar}`);
    }

    static staticMethod(): void {
        console.log("Inside static method");
    }
}

interface ITestSuite {
    readonly suiteId: number;    // Readonly property
    suiteName: string;
    testsCount?: number;         // Optional property
    [propName: string]: any;     // Index signature for additional properties

    // Method to describe the test suite
    describe(this: ITestSuite): void;

    // Method to execute the test suite
    runTests(): void;
}

/*
   This is a multi-line comment
   It can span multiple lines
*/

// Static method decorator
TestClass.staticMethod = funcDecorator(TestClass.staticMethod);

// Lambda expression
const square = (x: number): number => x * x;

// Multiline string
const multiLineStr: string = `
This is a
multi-line string
for testing purposes.
`;

const multilineFString: string = `This is a
multiline ${multiLineStr} string
spanning several lines
`;

const rawString: string = `This is a raw string with no special treatment for \\n`;

// List comprehension
const squaredNumbers: string[] = Array.from(Array(10).keys()).map(x => "x" + square(x));

// Set and Dictionary comprehension
const uniqueSquares: Set<number> = new Set(Array.from(Array(10).keys()).map(square));
const squaresDict: Record<number, number> = Object.fromEntries(Array.from(Array(10).keys()).map(x => [x, square(x)]));

// Exception handling
function exceptionHandling(x: number): void {
    try {
        if (x < 0) {
            throw new Error("Negative value");
        } else if (x === 0) {
            throw new Error("Division by zero");
        }
        const result = 10 / x;
    } catch (e) {
        console.error(`Caught an exception: ${e.message}`);
    } finally {
        console.log("This will always be printed");
    }
}

// Inplace operations
function inplaceOperations(): void {
    let x = 10;
    x += 5;
    x -= 3;
    x *= 2;
    x /= 4;
    console.log(`Inplace operations result: ${x}`);
}

// Control flow
function controlFlow(): void {
    if (testVar > 5) {
        console.log("testVar is greater than 5");
    } else {
        console.log("testVar is 5 or less");
    }

    let counter = 0;
    while (counter < 3) {
        console.log(`Counter is ${counter}`);
        counter++;
    }

    for (let i = 0; i < 3; i++) {
        console.log(`Loop iteration ${i}`);
    }

    // with statement equivalent in TypeScript
    try {
        const content = fs.readFileSync(__filename, 'utf8');
        console.log("Read from file:", content.split('\n')[0]);
    } catch (error) {
        console.error("Failed to read file");
    }
}

// Async function
async function asyncFunction(): Promise<void> {
    await sleep(1000);
    console.log("Async function executed");
}

// Main execution
if (require.main === module) {
    freeFunc();
    decoratedFunc();
    TestClass.classMethod();
    const instance = new TestClass();
    instance.instanceMethod();
    TestClass.staticMethod();
    console.log(square(5));
    exceptionHandling(0);
    inplaceOperations();
    controlFlow();
    (async () => await asyncFunction())();
}
