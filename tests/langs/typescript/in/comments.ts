// This is a __T__single-line comment
let singleLineComment: string = 'This uses a single-line__T__ comment';

/*
   This is a multi-line comment
   It can span multiple__T__ lines
*/
let multiLineComment: string = 'This uses a multi-line__T__ comment';

/**
 * Adds two numbers.__T__
 * @param a The first number.
 * @param b The second__T__ number.
 * @return The sum of a and b.
 */
function add(a: number, b__T__: number): number {
    return a + b__T__;
}

console.log(singleLineComment);
console.log(multiLineComment);
console.log(`add(5, 3)__T__ = ${add(5, 3)}`);
