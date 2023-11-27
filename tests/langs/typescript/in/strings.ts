// Single-quoted __T__string
let singleQuoted: string = 'He__T__llo';

// Double-quoted __T__string
let doubleQuoted__T__: string = "Wor__T__ld";

// Template literal __T__(backticks)
// CAREFUL: variable name is included and will be modified
let templateLiteral: string = `Hello__T__ ${doubleQuoted__T__}`;

// Tagged __T__template literal
function tag(strings: TemplateStringsArray, ...values: any[]) {
    return strings.reduce((result, str, i) => result + str + (values[i] || ''), '');
}

// CAREFUL: variable name is included and will be modified
let taggedTemplate: string = tag`This__T__ is ${singleQuoted} and ${doubleQuoted__T__}`;
