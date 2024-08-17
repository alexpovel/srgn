// Import necessary modules
import 'some/package';
import { promisify } from 'util';

const sleep = promisify(setTimeout);
// TypeScript Syntax Showcase

// Basic Types
let isDone: boolean = false;
let decimal: number = 6;
let color: string = "blue";
let list: number[] = [1, 2, 3];
let x: [string, number] = ["hello", 10];

// Enum
enum Color {Red, Green, Blue}
let c: Color = Color.Green;

// Any
let notSure: any = 4;
notSure = "maybe a string instead";
notSure = false;

// Void
function warnUser(): void {
    console.log("This is my warning message");

    try {
        throw new Error('This is an error');
    } catch (e) {
        console.error(e);
    } finally {
        console.log('finally');
    }
}

// Null and Undefined
let u: undefined = undefined;
let n: null = null; // Some comment

/* Some comment using different syntax */

// Never
function error(message: string): never {
    throw new Error(message);
}

// Object
declare function create(o: object | null): void;

// Type assertions
let someValue: any = "this is a string";
let strLength: number = (<string>someValue).length;
let strLength2: number = (someValue as string).length;

// Interface
interface LabelledValue {
    label: string;
    optional?: string;
    readonly x: number;
}

// Function Types
interface SearchFunc {
    (source: string, subString: string): boolean;
}

// Indexable Types
interface StringArray {
    [index: number]: string;
}

// Class Types
interface ClockInterface {
    currentTime: Date;
    setTime(d: Date): void;
}

class Clock implements ClockInterface {
    currentTime: Date = new Date();
    setTime(d: Date) {
        this.currentTime = d;
    }
    constructor(h: number, m: number) { }
}

// Extending Interfaces
interface Shape {
    color: string;
}

interface Square extends Shape {
    sideLength: number;
}

// Hybrid Types
interface Counter {
    (start: number): string;
    interval: number;
    reset(): void;
}

// Class
class Greeter {
    greeting: string;
    constructor(message: string) {
        this.greeting = message;
    }
    greet() {
        return "Hello, " + this.greeting;
    }
}

// Inheritance
class Animal {
    name: string;
    constructor(theName: string) { this.name = theName; }
    move(distanceInMeters: number = 0) {
        console.log(`${this.name} moved ${distanceInMeters}m.`);
    }
}

class Snake extends Animal {
    constructor(name: string) { super(name); }
    move(distanceInMeters = 5) {
        console.log("Slithering...");
        super.move(distanceInMeters);
    }
}

class Fish extends Animal {
    constructor(name: string) { super(name); }
    move(distanceInMeters = 5) {
        console.log("Swimming...");
        super.move(distanceInMeters);
    }

    swim() {
        console.log('swimming');
    }
}

// Public, private, and protected modifiers
class Person {
    protected name: string;
    constructor(name: string) { this.name = name; }
}

// Readonly modifier
class Octopus {
    readonly name: string;
    readonly numberOfLegs: number = 8;
    constructor (theName: string) {
        this.name = theName;
    }
}

// Accessors
class Employee {
    private _fullName: string;

    get fullName(): string {
        return this._fullName;
    }

    set fullName(newName: string) {
        this._fullName = newName;
    }
}

// Static Properties
class Grid {
    static origin = {x: 0, y: 0};
}

// Abstract Classes
abstract class Department {
    constructor(public name: string) {
    }
    printName(): void {
        console.log("Department name: " + this.name);
    }
    abstract printMeeting(): void;
}

// Generics
function identity<T>(arg: T): T {
    return arg;
}

// Generic Classes
class GenericNumber<T> {
    zeroValue: T;
    add: (x: T, y: T) => T;
}

// Generic Constraints
interface Lengthwise {
    length: number;
}

function loggingIdentity<T extends Lengthwise>(arg: T): T {
    console.log(arg.length);
    return arg;
}

// Using Type Parameters in Generic Constraints
function getProperty<T, K extends keyof T>(obj: T, key: K) {
    return obj[key];
}

// Union Types
function padLeft(value: string, padding: string | number) {
    // ...
}

// Type Guards
function isFish(pet: Fish | Snake): pet is Fish {
    return (<Fish>pet).swim !== undefined;
}

// Type Aliases
type Name = string;
type NameResolver = () => string;
type NameOrResolver = Name | NameResolver;

// String Literal Types
type Easing = "ease-in" | "ease-out" | "ease-in-out";

// Numeric Literal Types
function rollDice(): 1 | 2 | 3 | 4 | 5 | 6 {
    // ...
    return 1;
}

// Enum Member Types
enum ShapeKind {
    Circle,
    Square,
}

// Discriminated Unions
interface Square {
    kind: "square";
    size: number;
}
interface Rectangle {
    kind: "rectangle";
    width: number;
    height: number;
}
type OtherShape = Square | Rectangle;

// Index types
function pluck<T, K extends keyof T>(o: T, names: K[]): T[K][] {
  return names.map(n => o[n]);
}

// Mapped types
type Readonly<T> = {
    readonly [P in keyof T]: T[P];
}

// Conditional Types
type TypeName<T> =
    T extends string ? "string" :
    T extends number ? "number" :
    T extends boolean ? "boolean" :
    T extends undefined ? "undefined" :
    T extends Function ? "function" :
    "object";

// Decorators
function sealed(constructor: Function) {
    Object.seal(constructor);
    Object.seal(constructor.prototype);
}

@sealed
class Greeter2 {
    greeting: string;
    constructor(message: string) {
        this.greeting = message;
    }
    greet() {
        return "Hello, " + this.greeting;
    }
}

// Modules
export interface StringValidator {
    isAcceptable(s: string): boolean;
}

// Namespaces
namespace Validation {
    export interface StringValidator {
        isAcceptable(s: string): boolean;
    }
}

// JSX
declare namespace JSX {
    interface ElementClass {
        render: any;
    }
}

// Async/Await
async function getFoodItem(): Promise<string> {
    const result = await fetch('https://api.example.com/food');
    return result.json();
}

let foo = { bar: { baz: () => 42 } };

// Optional Chaining
let x2 = foo?.bar.baz();

// Nullish Coalescing
let x3 = foo ?? getFoodItem();

// BigInt
let big: bigint = 100n;

// const assertions
let x4 = "hello" as const;

// Template Literal Types
type World = "world";
type Greeting = `hello ${World}`;

// Raw string
String.raw`Hi\n${2+3}!`;

// Multi-line strings
let multiline = `This is a
multiline
string`;

// Key Remapping in Mapped Types
type MappedTypeWithNewKeys<T> = {
    [K in keyof T as KeyType]: T[K]
};

// Recursive Type Aliases
type JsonValue = string | number | boolean | null | JsonObject | JsonArray;
interface JsonObject { [key: string]: JsonValue }
interface JsonArray extends Array<JsonValue> {}

// Unknown
let notKnown: unknown = 4;

var isSomething: boolean = true;
