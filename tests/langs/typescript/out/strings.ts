// Single-quoted __T__string
let singleQuoted: string = 'Hello';

// Double-quoted __T__string
let doubleQuoted__T__: string = "World";

// Template literal __T__(backticks)
let templateLiteral: string = `Hello ${doubleQuoted__T__}`;

// Tagged __T__template literal
function tag(strings: TemplateStringsArray, ...values: any[]) {
    return strings.reduce((result, str, i) => result + str + (values[i] || ''), '');
}

let taggedTemplate: string = tag`This is ${singleQuoted} and ${doubleQuoted__T__}`;
