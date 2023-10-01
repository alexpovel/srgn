# betterletters

[`tr`][tr], with Unicode support and gimmicks, and a *scoping* concept.

## Pitch

TODO: Print help?

### Showcases

#### Expanding acronyms in docstrings (Python)

Imagine you [dislike acronyms and
abbreviations](https://www.instituteccp.com/a-case-against-abbreviations-and-acronyms/),
and would thus like to expand all of them. However, simple search-and-replace is
dangerous: actual code might be hit, rendering it syntactically invalid. Let's assume
we'd only like to hit documentation. The Python snippet

```python gnu.py
"""GNU module."""

def GNU_says_moo():
    """Make the GNU say moo."""

    GNU = """
      GNU
    """  # the GNU...

    print(GNU + " says moo")  # ...says moo
```

can be manipulated with an invocation of

```bash
cat gnu.py | betterletters --python 'doc-strings' 'GNU' 'GNU 🐂 is not Unix'
```

to read

```python output-gnu.py
"""GNU 🐂 is not Unix module."""

def GNU_says_moo():
    """Make the GNU 🐂 is not Unix say moo."""

    GNU = """
      GNU
    """  # the GNU...

    print(GNU + " says moo")  # ...says moo
```

No `gnu`s other than docstring ones were harmed in the process.

#### Converting `print` calls to proper `logging` (Python)

Say there's code making liberal use of `print`:

```python money.py
def print_money():
    """Let's print money 💸."""

    amount = 32
    print("Got here.")

    print_more = lambda s: print(f"Printed {s}")
    print_more(23)  # print the stuff

print_money()
print("Done.")
```

and a move to [`logging`](https://docs.python.org/3/library/logging.html) is desired.
That's fully automated by a call of

```bash
cat money.py | betterletters --python 'function-calls' '^print$' 'logging.info'
```

yielding

```python output-money.py
def print_money():
    """Let's print money 💸."""

    amount = 32
    logging.info("Got here.")

    print_more = lambda s: logging.info(f"Printed {s}")
    print_more(23)  # print the stuff

print_money()
logging.info("Done.")
```

Notice the [anchors](https://www.regular-expressions.info/anchors.html): `print_more` is
a function call as well, but `^print$` ensures it's not matched.

## Walkthrough

The tool is designed around **scopes** and **actions**. Scopes narrow down the parts of
the input to process. Actions then perform the processing. Generally, actions are
composable. Both are optional (but taking no action is pointless); specifying no scope
implies the entire input is in scope.

At the same time, there is considerable overlap with plain [`tr`][tr]: the tool is
designed to have close correspondence in the most common use cases, and only go beyond
when needed.

Let's start with actions first, as scopes will get more advanced.

### Actions

The simplest action is replacement. It is specially accessed (as an argument, not an
option) for compatibility with [`tr`][tr], and general ergonomics. All other actions are
given as flags.

#### Replacement

For example, simple, single-character replacements work as in [`tr`][tr]:

```console
$ echo 'Hello, World!' | betterletters 'H' 'J'
Jello, World!
```

The first argument is the scope (literal `H` in this case). Anything matched by it is
subject to processing (replacement by `J`, the second argument, in this case). However,
there is no direct concept of character classes. Instead, by default, the scope is a
regular expression pattern, so *its*
[classes](https://docs.rs/regex/1.9.5/regex/index.html#character-classes) can be used to
similar effect:

```console
$ echo 'Hello, World!' | betterletters '[a-z]' '_'
H____, W____!
```

The replacement occurs greedily across the entire match by default (note the [UTS
character class](https://docs.rs/regex/1.9.5/regex/index.html#ascii-character-classes),
reminiscent of [`tr`'s
`[:alnum:]`](https://github.com/coreutils/coreutils/blob/769ace51e8a1129c44ee4e7e209c3b2df2111524/src/tr.c#L322C25-L322C25)):

```console
$ echo 'ghp_oHn0As3cr3T!!' | betterletters 'ghp_[[:alnum:]]+' '*' # A GitHub token
*!!
```

However, in the presence of capture groups, the *individual characters comprising a
capture group match* are treated *individually* for processing, allowing a replacement
to be repeated:

```console
$ echo 'Hide ghp_th15 and ghp_th4t' | betterletters '(ghp_[[:alnum:]]+)' '*'
Hide ******** and ********
```

Advanced regex features are
[supported](https://docs.rs/fancy-regex/0.11.0/fancy_regex/index.html#syntax), for
example lookarounds:

```console
$ echo 'ghp_oHn0As3cr3T' | betterletters '(?<=ghp_)([[:alnum:]]+)' '*'
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

The tool is fully Unicode-aware, with useful support for [certain advanced
character
classes](https://github.com/rust-lang/regex/blob/061ee815ef2c44101dba7b0b124600fcb03c1912/UNICODE.md#rl12-properties):

```console
$ echo 'Mood: 🙂' | betterletters '🙂' '😀'
Mood: 😀
$ echo 'Mood: 🤮🤒🤧🦠 :(' | betterletters '\p{Emoji_Presentation}' '😷'
Mood: 😷😷😷😷 :(
```

#### Beyond replacement

Seeing how the replacement is merely a static string, its usefulness is limited. This is
where [`tr`'s secret sauce](https://maizure.org/projects/decoded-gnu-coreutils/tr.html)
ordinarily comes into play: using its character classes, which are valid in the second
position as well, neatly translating from members of the first to the second. Here,
those classes are instead regexes, and only valid in first position (the scope). A
regular expression being a state machine, it is impossible to match onto a 'list of
characters', which in `tr` is the second (optional) argument. That concept is out the
window, and its flexibility lost.

Instead, the offered actions, all of them **fixed**, are used. A peek at [the most
common use cases for `tr`](#common-tr-use-cases) reveals that the provided set of
actions covers virtually all of them! Feel free to file an issue if your use case is not
covered.

Onto the next action.

#### Deletion

Removes whatever is found from the input. Same flag name as in `tr`.

```console
$ echo 'Hello, World!' | betterletters -d '(H|W|!)'
ello, orld
```

As the default scope is to match the entire input, it is an error to specify deletion
without a scope.

#### Squeezing

Squeezes repeats of characters matching the scope into single occurrences. Same flag
name as in `tr`.

```console
$ echo 'Helloooo Woooorld!!!' | betterletters -s '(o|!)'
Hello World!
```

If a character class is passed, all members of that class are squeezed into whatever
occurred first:

```console
$ echo 'The number is: 3490834' | betterletters -s '\d'
The number is: 3
```

Greediness in matching is not modified, so take care:

```console
$ echo 'Winter is coming... 🌞🌞🌞' | betterletters -s '🌞+'
Winter is coming... 🌞🌞🌞
```

The pattern matched the entire string of suns, so there's nothing to squeeze. Summer
prevails.

Again, as with [deletion](#deletion), specifying squeezing without an *explicit* scope
is an error. Squeezing the entire input is probably not what you want.

#### Character casing

A good chunk of `tr` usage [falls into this category](#changing-character-casing). It's
very straightforward.

```console
$ echo 'Hello, World!' | betterletters --lower
hello, world!
$ echo 'Hello, World!' | betterletters --upper
HELLO, WORLD!
$ echo 'hello, world!' | betterletters --titlecase
Hello, World!
```

#### Normalization

Decomposes input according to [Normalization Form
D](https://en.wikipedia.org/wiki/Unicode_equivalence#Normal_forms), and then discards
code points of the [Mark
category](https://en.wikipedia.org/wiki/Unicode_character_property#General_Category)
(see [examples](https://www.compart.com/en/unicode/category/Mn)). That roughly means:
take fancy character, rip off dangly bits, throw those away.

```console
$ echo 'Naïve jalapeño ärgert mgła' | betterletters -d '\P{ASCII}' # Naive approach
Nave jalapeo rgert mga
$ echo 'Naïve jalapeño ärgert mgła' | betterletters --normalize # Normalize is smarter
Naive jalapeno argert mgła
```

Notice how `mgła` is out of scope for NFD, as it is not decomposable (at least that's
what ChatGPT whispers in my ear).

#### Symbols

This action replaces multi-character ASCII symbols with their single-code point native
Unicode counterparts.

```console
$ echo '(A --> B) != C --- obviously' | betterletters --symbols
(A ⟶ B) ≠ C — obviously
```

Alternatively, if you're only interested in math, make use of scoping:

```console
$ echo 'A <= B --- More is--obviously--possible' | betterletters --symbols '<='
A ≤ B --- More is--obviously--possible
```

As there is a [1:1 correspondence](https://en.wikipedia.org/wiki/Bijection) between an
ASCII symbol and its replacement, the effect is reversible[^1]:

```console
$ echo 'A ⇒ B' | betterletters --symbols --invert
A => B
```

There is only a limited set of symbols supported as of right now, but more can be added.

#### German

This action replaces alternative spellings of German special characters (ae, oe, ue, ss)
with their native versions (ä, ö, ü, ß).

```console
$ echo 'Gruess Gott, Poeten und Abenteuergruetze!' | betterletters --german
Grüß Gott, Poeten und Abenteuergrütze!
```

This action is based on a word list. Note:

- empty scope and replacement: the entire input will be processed, and no replacement is
  performed,
- `Poeten` remained as-is, instead of being naively converted to `Pöten`,
- as a (compound) word, `Abenteuergrütze` is not going to be found in [any reasonable
  word list](https://www.duden.de/suchen/dudenonline/Stinkegr%C3%BCtze), but was handled
  properly nonetheless.

### Combining Actions

Most actions are composable, unless it would not make sense (like for
[deletion](#deletion)). Their order of application is fixed, so the *order* of the flags
given has no influence. Replacements always occur first. Generally, the CLI is designed
to prevent misuse and surprises: it prefers crashing to doing something unexpected. Note
that lots of combinations *are* technically possible, but might yield nonsensical
results.

```console
$ echo 'Koeffizienten != Bruecken...' | betterletters -Sgu
KOEFFIZIENTEN ≠ BRÜCKEN...
```

A more narrow scope can be specified, and will apply to *all* actions equally:

```console
$ echo 'Koeffizienten != Bruecken...' | betterletters -Sgu '\b\w{1,8}\b'
Koeffizienten != BRÜCKEN...
```

The [word boundaries](https://www.regular-expressions.info/wordboundaries.html) are
required as otherwise `Koeffizienten` is matched as `Koeffizi` and `enten`. Note how the
trailing periods cannot be, for example, squeezed. The required scope of `\.` would
interfere with the given one. Regular piping solves this:

```console
$ echo 'Koeffizienten != Bruecken...' | betterletters -Sgu '\b\w{1,8}\b' | betterletters -s '\.'
Koeffizienten != BRÜCKEN.
```

The specially treated replacement action is also composable:

```console
$ echo 'Mooood: 🤮🤒🤧🦠!!!' | betterletters -s '\p{Emoji}' '😷'
Mooood: 😷!!!
```

Emojis are first all replaced, then squeezed. Notice how nothing else is squeezed.

### Scopes

<!-- TODO -->

## Rust library

<!-- TODO -->

## Contributing

<!-- TODO -->

## Common `tr` use cases

In theory, `tr` is quite flexible. In practice, it is commonly used mainly across a
couple specific tasks. Next to its two positional arguments ('arrays of characters'),
one finds four flags:

1. `-c`, `-C`, `--complement`: complement the first array
2. `-d`, `--delete`: delete characters in the first first array
3. `-s`, `--squeeze-repeats`: squeeze repeats of characters in the first array
4. `-t`, `--truncate-set1`: truncate the first array to the length of the second

In this tool, these are implemented as follows:

1. is not available directly as an option; instead, negation of regular expression
   classes can be used (e.g., `[^a-z]`), to much more potent, flexible and well-known
   effect
2. available (via regex)
3. available (via regex)
4. not available: it's inapplicable to regular expressions, not commonly used and, if
   used, often misused

To show how uses of `tr` found in the wild can translate to this tool, consider the
following section.

### Use cases and equivalences in this tool

The following sections are the approximate categories much of `tr` usage falls into.
They were found using [GitHub's code search](https://cs.github.com). The corresponding
queries are given. Results are from the first page of results at the time. The code
samples are links to their respective sources.

As the stdin isn't known (usually dynamic), some representative samples are used and the
tool is exercised on those.

#### Identifier Safety

Making inputs safe for use as identifiers, for example as variable names.

[Query](https://github.com/search?type=code&q=%22tr+-c%22)

1. [`tr -C '[:alnum:]_\n'
   '_'`](https://github.com/grafana/grafana/blob/9328fda8ea8384e8cfcf1c78d1fe95d92bbad786/docs/make-docs#L234)

    Translates to:

    ```console
    $ echo 'some-variable? 🤔' | betterletters '[^[:alnum:]_\n]' '_'
    some_variable___
    ```

    Similar examples are:

    - [`tr -C "[:alnum:]"
      '-'`](https://github.com/elastic/go-elasticsearch/blob/594de0c207ef5c4804615ebedd043a789ef3ce75/.buildkite/functions/imports.sh#L38)
    - [`tr -C "A-Za-z0-9"
      "_"`](https://github.com/Homebrew/brew/blob/b2cf50bbe10ab996a1e3365545fadabf36df777a/Library/Homebrew/cmd/update.sh#L104)
    - [`tr -c '[a-zA-Z0-9-]'
      '-'`](https://github.com/xamarin/xamarin-macios/blob/c14f9ff7c7693ab060c4b84c78075ff975ea7c64/Make.config#L69)
    - [`tr -C 'a-zA-Z0-9@_'
      '_'`](https://github.com/openzfsonwindows/openzfs/blob/61f4ce826122f19a0a0c734efb4c2469b2aa367b/autogen.sh#L22)

2. [`tr -c '[:alnum:]'
   _`](https://github.com/freebsd/freebsd-src/blob/9dc0c983b0931f359c2ff10d47ad835ef74e929a/libexec/rc/rc.d/jail#L413)

    Translates to:

    ```console
    $ echo 'some  variablê' | betterletters '[^[:alnum:]]' '_'
    some__variabl_
    ```

3. [`tr -c -s '[:alnum:]'
   '-'`](https://github.com/weaviate/weaviate/blob/169381df70852ef687528ebf81e27869b3017403/ci/push_docker.sh#L26)

    Translates to:

    ```console
    $ echo '🙂 hellö???' | betterletters -s '[^[:alnum:]]' '-'
    -hell-
    ```

#### Literal-to-literal translation

Translates a *single*, literal character to another, for example to clean newlines.

[Query](https://github.com/search?q=%22%7C+tr++%22+%28path%3A*.sh+OR+path%3A*.yml+OR+path%3A*.yaml%29&type=code&ref=advsearch)

1. [`tr  " "
   ";"`](https://github.com/facebook/react-native/blob/d31d16b19cecb893a388fcb141602e8abad4aa76/packages/react-native/sdks/hermes-engine/utils/build-hermes-xcode.sh#L32)

    Translates to:

    ```console
    $ echo 'x86_64 arm64 i386' | betterletters ' ' ';'
    x86_64;arm64;i386
    ```

    Similar examples are:

    - [`tr ' '
      '-'`](https://github.com/eyedol/tools/blob/e940fe4484b486aa8d42a76d9305a9227bea7552/backup.sh#L11)
    - [`tr '-'
      '_'`](https://github.com/rerun/rerun/blob/aa5ad6360780ddbbb11835654e8f49b3827f15cd/modules/stubbs/lib/functions.sh#L147)

2. [`tr '.'
   "\n"`](https://github.com/SDA-SE/cluster-image-scanner/blob/a769be53eae423f57a7f34c429cfa3a2770a859e/images/scan/syft/build.sh#L16):

    Translates to:

    ```console
    $ echo '3.12.1' | betterletters --literal-string '.' '\n'  # Escape sequence works
    3
    12
    1
    $ echo '3.12.1' | betterletters '\.' '\n'  # Escape regex otherwise
    3
    12
    1
    ```

3. [`tr '\n'
   ','`](https://github.com/gtoubassi/dqn-atari/blob/513b307039f4c28b5b517cd542ad625b41f0ef50/logstats.sh#L43)

    Translates to:

    ```console
    $ echo -ne 'Some\nMulti\nLine\nText' | betterletters --literal-string '\n' ','
    Some,Multi,Line,Text
    ```

    If escape sequences remain uninterpreted (`echo -E`, the default), the scope's
    escape sequence will need to be turned into a literal `\` and `n` as well, as it is
    otherwise interpreted by the tool as a newline:

    ```console
    $ echo -nE 'Some\nMulti\nLine\nText' | betterletters --literal-string '\\n' ','
    Some,Multi,Line,Text
    ```

    Similar examples are:

    - [`tr '\n' '
      '`](https://github.com/mhassan2/splunk-n-box/blob/a721af8b8ae6103a7b274651206d4812d37db398/scripts/viz.sh#L427)
    - [`tr "\n" "
      "`](https://github.com/ministryofjustice/modernisation-platform-configuration-management/blob/6a1dd9f31a62d68d796ae304165eed1fcb1b822e/ansible/roles/nomis-weblogic/tasks/patch-weblogic.yml#L13)

#### Removing a character class

Very useful to remove whole categories in one fell swoop.

[Query](https://github.com/search?q=%22%7C+tr++%22+%28path%3A*.sh+OR+path%3A*.yml+OR+path%3A*.yaml%29&type=code&ref=advsearch)

1. [`tr -d
   '[:punct:]'`](https://github.com/CNMAT/OpenSoundControl.org/blob/fb7b3b48ba9ac64eae030e3333f9a980f4f8fd59/build-implementations.sh#L98)
   which they [describe
   as](https://github.com/CNMAT/OpenSoundControl.org/blob/fb7b3b48ba9ac64eae030e3333f9a980f4f8fd59/build-implementations.sh#L94):

    > Omit all punctuation characters

    translates to:

    ```console
    $ echo 'Lots... of... punctuation, man.' | betterletters -d '[[:punct:]]'
    Lots of punctuation man
    ```

Lots of use cases also call for **inverting**, then removing a character class.

[Query](https://github.com/search?type=code&q=%22tr+-c%22)

1. [`tr
    -cd a-z`](https://github.com/git/git/blob/d6c51973e4a0e889d1a426da08f52b9203fa1df2/t/lib-credential.sh#L542)

    Translates to:

    ```console
    $ echo 'i RLY love LOWERCASING everything!' | betterletters -d '[^[:lower:]]'
    iloveeverything
    ```

2. [`tr -cd
   'a-zA-Z0-9'`](https://github.com/gitlabhq/gitlabhq/blob/e74bf51e817ee50e85b1bbdc34f0443d1088fd68/doc/user/project/service_desk/configure.md?plain=1#L553)

    Translates to:

    ```console
    $ echo 'All0wed ??? 💥' | betterletters -d '[^[:alnum:]]'
    All0wed
    ```

3. [`tr -cd
   '[[:digit:]]'`](https://github.com/coredns/coredns/blob/b5e6291115d1e60fed561c64d70341b354e69504/Makefile.release#L94)

   Translates to:

    ```console
    $ echo '{"id": 34987, "name": "Harold"}' | betterletters -d '[^[:digit:]]'
    34987
    ```

#### Remove literal character(s)

Identical to replacing them with the empty string.

[Query](https://github.com/search?q=%22%7C+tr+%22&type=code&ref=advsearch)

1. [`tr -d "."`](https://github.com/ohmyzsh/ohmyzsh/blob/079dbff2c4f22935a71101c511e2285327d8ab68/themes/gallois.zsh-theme#L82)

    Translates to:

    ```console
    $ echo '1632485561.123456' | betterletters -d '\.'  # Unix timestamp
    1632485561123456
    ```

    Similar examples are:

    - [`tr -d '\`'`](https://github.com/jlevy/the-art-of-command-line/blob/6b50745d2e788add2e8f1ed29010e72659a9a074/README.md?plain=1#L22)
    - [`tr -d ' '`](https://github.com/hfg-gmuend/openmoji/blob/9782be9d240513a3d609a4bd6f1176f2d7e1b804/helpers/lib/optimize-build.sh#L77)
    - [`tr -d ' '`](https://github.com/PaNOSC-ViNYL/SimEx/blob/0ca295ec57864c0e468eba849d3f44f992c59634/Docker/simex_devel/simex_install.sh#L44)
2. [`tr -d '\r\n'`](https://github.com/kubernetes/kubernetes/blob/37cf2638c975080232990d2fc2c24d0f40c38074/cluster/gce/util.sh#L1690)

    Translates to:

    ```console
    $ echo -e 'DOS-Style\r\n\r\nLines' | betterletters -d '\r\n'
    DOS-StyleLines
    ```

    Similar examples are:

    - [`tr -d '\r'`](https://github.com/mhassan2/splunk-n-box/blob/a721af8b8ae6103a7b274651206d4812d37db398/scripts/viz.sh#L427)
    - [`tr -d '\r'`](https://github.com/Cidaas/cidaas-shopware-connect-plugin/blob/519e21a9a385b26803ec442e2a8b59918a948f77/.gitlab-ci.yml#L43)

#### Squeeze whitespace

Remove repeated whitespace, as it often occurs when slicing and dicing text.

[Query](https://github.com/search?type=code&q=%22+tr+-s%22)

1. [`tr -s '[:space:]'`](https://github.com/ohmyzsh/ohmyzsh/blob/bbda81fe4b338f00bbf7c7f33e6d1b12d067dc05/plugins/alias-finder/alias-finder.plugin.zsh#L26)

    Translates to:

    ```console
    $ echo 'Lots   of  space !' | betterletters -s '[[:space:]]'  # Single space stays
    Lots of space !
    ```

    Similar examples are:

   - [`tr -s " "`](https://github.com/facebookresearch/fastText/blob/166ce2c71a497ff81cb62ec151be5b569e1f1be6/.circleci/pull_data.sh#L18)
   - [`tr -s
     [:blank:]`](https://github.com/Atmosphere-NX/Atmosphere/blob/4fe9a89ab8ed958a3e080d7ee11767bef9cb2d57/atmosphere.mk#L11)
     (`blank` is `\t` and space)
   - [`tr
     -s`](https://github.com/kovidgoyal/kitty/blob/863adb3e8d8e7229610c6b0e6bc8d48db9becda5/kitty/rc/get_colors.py#L28)
     (no argument: this will error out; presumably space was meant)
   - [`tr -s ' '`](https://github.com/google/jax/blob/bf40f75bd59501d66a0e500d255daca3f9f2895e/build/rocm/build_rocm.sh#L65)
   - [`tr -s ' '`](https://github.com/chromium/chromium/blob/ffed2601d91f4413ca4672a6027c2c05d49df815/docs/linux/minidump_to_core.md?plain=1#L111)
   - [`tr -s '[:space:]'`](https://github.com/ceph/ceph/blob/6c387554d8e104727b3e448a1def4f1991be1ff7/src/stop.sh#L188)
   - [`tr -s ' '`](https://github.com/PyO3/pyo3/blob/8f4a26a66ecee4cfa473ada8cb27a57c4533d04f/.netlify/build.sh#L7)
2. [`tr -s ' '
   '\n'`](https://github.com/doocs/leetcode/blob/4f89e08ed45f7d5e1047767071001073ffe4d32b/solution/0100-0199/0192.Word%20Frequency/README.md?plain=1#L54)
   (squeeze, *then replace*)

    Translates to:

    ```console
    $ echo '1969-12-28    13:37:45Z' | betterletters -s ' ' 'T'  # ISO8601
    1969-12-28T13:37:45Z
    ```

3. [`tr -s '[:blank:]' ':'`](https://github.com/cockroachdb/cockroach/blob/985662236d7bf273b93a7b5e32def8e2d1043640/docs/generated/http/BUILD.bazel#L76)

    Translates to:

    ```console
    $ echo -e '/usr/local/sbin \t /usr/local/bin' | betterletters -s '[[:blank:]]' ':'
    /usr/local/sbin:/usr/local/bin
    ```

#### Changing character casing

A straightforward use case. Upper- and lowercase are often used.

[Query](https://github.com/search?q=%22%7C+tr++%22+%28path%3A*.sh+OR+path%3A*.yml+OR+path%3A*.yaml%29&type=code&ref=advsearch)

1. [`tr A-Z
   a-z`](https://github.com/golang/go/blob/a742ae493ff59a71131706500ce53f85477897f0/src/encoding/xml/xml.go#L1874)
   (lowercasing)

    Translates to:

    ```console
    $ echo 'WHY ARE WE YELLING?' | betterletters --lower
    why are we yelling?
    ```

    Notice the default scope. It can be refined to lowercase only long words, for
    example:

    ```console
    $ echo 'WHY ARE WE YELLING?' | betterletters --lower '\b\w{,3}\b'
    why are we YELLING?
    ```

    Similar examples are:

    - [`tr 'A-Z' 'a-z'`](https://github.com/nwchemgit/nwchem/blob/aad4ecd5657055b085b57115314e4d56271ad749/travis/guess_simd.sh#L13)
    - [`tr '[A-Z]' '[a-z]'`](https://github.com/XIMDEX/xcms/blob/4dd3c055de5cb0eebed28f1e9da87ed731a44a99/bin/lib/util.sh#L47)
    - [`tr '[A-Z]' '[a-z]'`](https://github.com/varunjampani/video_prop_networks/blob/4f4a39842bd9112932abe40bad746c174a242bf6/lib/davis/configure.sh#L30)
    - [`tr '[:upper:]' '[:lower:]'`](https://github.com/PaNOSC-ViNYL/SimEx/blob/0ca295ec57864c0e468eba849d3f44f992c59634/Docker/simex_devel/simex_install.sh#L44)
    - [`tr "[:upper:]" "[:lower:]"`](https://github.com/tst-labs/esocial/blob/b678f59bba883a63e91be79d7f2853a57156cf7b/src/esocial-esquemas/generate-java-from-xsd.sh#L11)
2. [`tr '[a-z]'
   '[A-Z]'`](https://github.com/henrikpersson/potatis/blob/63feb9de28781e4e9c62bd091bd335b87b474cb1/nes-android/install.sh#L10)
   (uppercasing)

    Translates to:

    ```console
    $ echo 'why are we not yelling?' | betterletters --upper
    WHY ARE WE NOT YELLING?
    ```

    Similar examples are:

    - [`tr '[a-z]' '[A-Z]'`](https://github.com/basho/riak-zabbix/blob/423e21c31821a345bf59ec4b2baba06d532a7f30/build_templates.sh#L40)
    - [`tr "[:lower:]" "[:upper:]"`](https://github.com/Fivium/Oracle-Backup-and-Sync/blob/036aace4a8eb45ab7e6e226ddceb08f35c46b9f3/dbsync/scripts/dbsync.sh#L122)
    - [`tr "[:lower:]" "[:upper:]"`](https://github.com/jorgeazevedo/xenomai-lab/blob/a2ce85a86f37fd9762905026ce4a1542684c714b/data/.xenomailab/blocks/template/rename.sh#L5)

[tr]: https://www.gnu.org/software/coreutils/manual/html_node/tr-invocation.html
[^1]: Currently, reversibility is not possible for any other action. For example,
    lowercasing is not the inverse of uppercasing. Information is lost, so it cannot be
    undone. Structure (imagine mixed case) was lost. Something something entropy...
