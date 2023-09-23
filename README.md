# betterletters

`tr`, with Unicode support and gimmicks, and a *scoping* concept.

## Usage

### As a drop-in replacement for `tr`

There is some overlap with plain `tr`, so simple replacements work as expected:

```console
$ echo 'Hello, World!' | betterletters 'H' 'J'
Jello, World!
```

However, there is no direct concept of character classes. Instead, the first argument is
a regular expression pattern, so *its* classes can be used to similar effect:

```console
$ echo 'Hello, World!' | betterletters '[a-z]' '_'
H____, W____!
```

The replacement occurs greedily across the entire match by default:

```console
$ echo 'ghp_oHn0As3cr3T' | betterletters 'ghp_[a-zA-Z0-9]+' '*' # A GitHub token
*
```

However, in the presence of capture groups, the individual characters comprising a
capture group match are treated *individually* for processing, allowing a replacement to
be repeated:

```console
$ echo 'ghp_oHn0As3cr3T' | betterletters '(ghp_[a-zA-Z0-9]+)' '*'
***************
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
