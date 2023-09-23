# betterletters

`tr`, with Unicode support and gimmicks, and a *scoping* concept.

## Usage

1. Conventional
   1. Actions
2. Unicode tricks
3. Scoping

### Conventional

There is considerable overlap with plain `tr`: the tool is designed to have close to
drop-in compatibility for the most common use cases. As such, the tool can be used
'conventionally'.

#### Replacements

For example, simple replacements work as expected:

```console
$ echo 'Hello, World!' | betterletters 'H' 'J'
Jello, World!
```

However, there is no direct concept of character classes. Instead, the first argument is
a regular expression pattern, so *its* [classes](https://docs.rs/regex/1.9.5/regex/index.html#character-classes) can be used to similar effect:

```console
$ echo 'Hello, World!' | betterletters '[a-z]' '_'
H____, W____!
```

The replacement occurs greedily across the entire match by default (note the [UTS character
class](https://docs.rs/regex/1.9.5/regex/index.html#ascii-character-classes),
reminiscent of [`tr`'s
`[:alnum:]`](https://github.com/coreutils/coreutils/blob/769ace51e8a1129c44ee4e7e209c3b2df2111524/src/tr.c#L322C25-L322C25)):

```console
$ echo 'ghp_oHn0As3cr3T!!' | betterletters 'ghp_[[:alnum:]]+' '*' # A GitHub token
*!!
```

However,
in the presence of capture groups, the *individual characters comprising a capture group
match* are treated *individually* for processing, allowing a replacement to be repeated:

```console
$ echo 'Hide ghp_th15 and ghp_m0r3' | betterletters '(ghp_[a-zA-Z0-9]+)' '*'
Hide ******** and ********
```

Advanced regex features are
[supported](https://docs.rs/fancy-regex/0.11.0/fancy_regex/index.html#syntax), for
example lookarounds:

```console
$ echo 'ghp_oHn0As3cr3T' | betterletters '(?<=ghp_)([a-zA-Z0-9]+)' '*'
ghp_***********
```

Take care in using these safely, as advanced patterns come without certain [safety and
performance guarantees](https://docs.rs/regex/latest/regex/#untrusted-input). If they
aren't used, [performance is not
impacted](https://docs.rs/fancy-regex/0.11.0/fancy_regex/index.html#).

The replacement is not limited to a single character. It can be any string, for example
to fix [this quote](http://regex.info/blog/2006-09-15/247):

```console
$ echo '"Using regex, I now have no issues."' | betterletters 'no issues' '2 problems'
"Using regex, I now have 2 problems."
```

#### Other actions

Seeing how the replacement is merely a static string, its usefulness is limited. This is
where [`tr`'s secret sauce](https://maizure.org/projects/decoded-gnu-coreutils/tr.html)
comes into play using its character classes, which are valid in the second position as
well, neatly translating from members of the first to the second. Here, those classes
are instead regexes, and only valid in first position. A regular expression being a
state machine, it is impossible to match onto a 'list of characters'. That concept is
out the window, and its flexibility lost.

## Common `tr` use cases

In theory, `tr` is quite flexible. In practice, it is commonly used mainly across a
couple specific tasks. Next to its two positional arguments ('arrays of characters'),
one finds four flags:

1. `-c`, `-C`, `--complement`: complement the first array
2. `-d`, `--delete`: delete characters in the first first array
3. `-s`, `--squeeze-repeats`: squeeze repeats of characters in the first array
4. `-t`, `--truncate-set1`: truncate the first array to the length of the second

In this tool, these are implemented as follows:

1. is not available directly as an option; instead, negation of regular expression classes can be used (e.g., `[^a-z]`), to much more potent, flexible and well-known effect
2. available (via regex)
3. available (via regex)
4. not available: it's inapplicable to regular expressions, not commonly used and, if used, often misused

To show how uses of `tr` found in the wild can translate to this tool, consider the
following section.

### Use cases and equivalences in this tool

The following sections are the approximate categories much of `tr` usage falls into.
They were found using [GitHub's code search](https://cs.github.com). The corresponding
queries are given. Results are from the first page of results at the time.

As the stdin isn't known (usually dynamic), some representative samples are used and the
tool is exercised on those.

#### Identifier Safety

Making inputs safe for use as identifiers, for example as variable names.

[Query](https://github.com/search?type=code&q=%22tr+-c%22)

1. [`tr -C '[:alnum:]_\n' '_'`](https://github.com/grafana/grafana/blob/9328fda8ea8384e8cfcf1c78d1fe95d92bbad786/docs/make-docs#L234)

   Translates to:

   ```console
   $ echo 'some-variable? 🤔' | betterletters '[^[[:alnum:]]_\n]' '_'
    some_variable___
   ```

2. [`tr -c '[:alnum:]' _`](https://github.com/freebsd/freebsd-src/blob/9dc0c983b0931f359c2ff10d47ad835ef74e929a/libexec/rc/rc.d/jail#L413)

    Translates to:

    ```console
    $ echo 'some  variablê' | betterletters '[^[[:alnum:]]]' '_'
    some__variabl_
    ```
