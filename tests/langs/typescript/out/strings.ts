// Single-quoted __T__string
let singleQuoted: string = 'Hello';

// Double-quoted __T__string
let doubleQuoted__T__: string = "World";

// Template literal __T__(backticks)
// CAREFUL: variable name is included and will be modified
let templateLiteral: string = `Hello ${doubleQuoted}`;

// Tagged __T__template literal
function tag(strings: TemplateStringsArray, ...values: any[]) {
    return strings.reduce((result, str, i) => result + str + (values[i] || ''), '');
}

// CAREFUL: variable name is included and will be modified
let taggedTemplate: string = tag`This is ${singleQuoted} and ${doubleQuoted}`;
