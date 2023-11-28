// Single-quoted __T__string
let singleQuoted: string = 'He__T__llo';

// Double-quoted __T__string
let doubleQuoted__T__: string = "Wor__T__ld";

// Template literal __T__(backticks)
let templateLiteral: string = `Hello__T__ ${doubleQuoted__T__}`;

// Tagged __T__template literal
function tag(strings: TemplateStringsArray, ...values: any[]) {
    return strings.reduce((result, str, i) => result + str + (values[i] || ''), '');
}

let taggedTemplate: string = tag`This__T__ is ${singleQuoted} and ${doubleQuoted__T__}`;
