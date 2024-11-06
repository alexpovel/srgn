//! This module contains a small bash interpreter, capable of interpreting simple pipes
//! of programs such as `echo 'Hello World!' | some-program --some-option | some-program
//! arg1 arg2`. It knows how to process and forward stdin and stdout, and will assert
//! commands run terminate as expected, with stdout as expected.
//!
//! There is also a small facility for tracking files, enabling usage of `cat some-file`
//! as stdin for later processes.
//!
//! The interpreter can be used to run some binary under test on provided input and test
//! against expected output. In this case, the source for all these elements is a
//! Markdown document.

#[cfg(all(test, feature = "all"))]
mod tests {
    use core::{fmt, panic};
    use std::cell::RefCell;
    use std::collections::{HashMap, VecDeque};
    use std::io::Write;
    use std::mem::ManuallyDrop;
    use std::rc::Rc;

    use assert_cmd::Command;
    use comrak::nodes::{NodeCodeBlock, NodeValue};
    use comrak::{parse_document, Arena, ComrakOptions};
    use fancy_regex::Regex;
    use itertools::Itertools;
    use nom::branch::alt;
    use nom::bytes::complete::{escaped, tag, take_until, take_while1};
    use nom::character::complete::{
        alpha1 as ascii_alpha1, alphanumeric1 as ascii_alphanumeric1, anychar, char, line_ending,
        none_of, space0, space1,
    };
    use nom::combinator::{cut, eof, map, opt};
    use nom::error::ParseError;
    use nom::multi::{many0, many1, many_till, separated_list1};
    use nom::sequence::{delimited, preceded, tuple};
    use nom::{Finish, IResult};
    use pretty_assertions::assert_eq;
    use tempfile::NamedTempFile;
    use unescape::unescape;

    const PROGRAM_NAME: &str = env!("CARGO_PKG_NAME");
    const DOCUMENT: &str = include_str!("../README.md");

    /// A flag, either short or long.
    ///
    /// Does not have a value, e.g. `--flag` or `-f`.
    #[derive(Debug, Clone, PartialEq, Eq)]
    enum Flag {
        Short(char),
        Long(String),
    }

    impl From<char> for Flag {
        fn from(c: char) -> Self {
            Self::Short(c)
        }
    }

    impl From<&str> for Flag {
        fn from(s: &str) -> Self {
            Self::Long(s.to_string())
        }
    }

    impl fmt::Display for Flag {
        fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
            match self {
                Self::Short(c) => write!(f, "-{c}"),
                Self::Long(s) => write!(f, "--{s}"),
            }
        }
    }

    /// An option, either short or long.
    ///
    /// Has a value, e.g. `--option value` or `-o value`.
    #[derive(Debug, Clone, PartialEq, Eq)]
    enum Opt {
        #[allow(dead_code)] // Not used yet
        Short(char, String),
        Long(String, String),
    }

    impl From<(&str, &str)> for Opt {
        fn from((s, v): (&str, &str)) -> Self {
            Self::Long(s.to_string(), v.to_string())
        }
    }

    /// A positional argument.
    #[derive(Debug, Clone, PartialEq, Eq)]
    struct Arg(String);

    impl From<&str> for Arg {
        fn from(s: &str) -> Self {
            Self(s.to_string())
        }
    }

    impl fmt::Display for Arg {
        fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
            write!(f, "{}", self.0)
        }
    }

    /// A collected, whole invocation of a program, including all bits and pieces
    /// required for running *except* the program name itself.
    #[derive(Debug, Clone, PartialEq, Eq, Default)]
    struct Invocation {
        flags: Vec<Flag>,
        opts: Vec<Opt>,
        args: Vec<Arg>,
        //
        stdin: Option<String>,
        stdout: Option<String>,
    }

    #[derive(Debug, Clone, PartialEq, Eq)]
    enum Program {
        /// The `echo` program, used to generate stdin for the program under test.
        Echo(Invocation),
        /// The `cat` program, used to generate stdin for the program under test.
        Cat(Invocation),
        /// The binary under test itself.
        Self_(Invocation),
    }

    impl Program {
        fn from_name(name: &str, mut invocation: Invocation) -> Self {
            match name {
                "echo" => {
                    if invocation.flags.contains(&Flag::Short('e')) {
                        invocation.args = invocation
                            .args
                            .into_iter()
                            .map(|a| Arg(unescape(a.0.as_str()).expect("Invalid escape sequence")))
                            .collect();
                    }

                    Self::Echo(invocation)
                }
                "cat" => Self::Cat(invocation),
                PROGRAM_NAME => Self::Self_(invocation),
                _ => panic!("Unsupported program name: {}", name),
            }
        }

        const fn name(&self) -> &str {
            match self {
                Self::Echo(_) => "echo",
                Self::Cat(_) => "cat",
                Self::Self_(_) => PROGRAM_NAME,
            }
        }

        fn stdout(&self) -> Option<String> {
            match self {
                Self::Echo(inv) | Self::Cat(inv) | Self::Self_(inv) => inv.stdout.clone(),
            }
        }

        /// This sets an option to forcibly ignore any provided stdin.
        ///
        /// Even though a command might run with `None` for its stdin, the binary under
        /// test might heuristically check for a readable stdin and somehow detect it.
        /// This might be a quirk in the command execution from [`Command`]... no idea.
        /// For that scenario, we need a hacky method to disable stdin for good.
        fn force_ignore_stdin(&mut self) {
            match self {
                Self::Self_(inv) => inv
                    .opts
                    .push(Opt::Long("stdin-override-to".into(), "false".into())),
                _ => panic!("Forcing stdin ignore only applicable to `self` program"),
            }
        }
    }

    impl TryFrom<Program> for Command {
        type Error = &'static str;

        fn try_from(prog: Program) -> Result<Self, Self::Error> {
            let name = prog.name().to_string();

            match prog {
                Program::Echo(_) | Program::Cat(_) => {
                    Err("Cannot be run, only used to generate stdin")
                }
                Program::Self_(inv) => {
                    let mut cmd = Self::cargo_bin(name).expect("Should be able to find binary");

                    for flag in &inv.flags {
                        cmd.arg(flag.to_string());
                    }

                    for opt in inv.opts {
                        match opt {
                            Opt::Short(name, value) => {
                                // Push these separately, as `arg` will escape the
                                // value, and something like `--option value` will be
                                // taken as a single arg, breaking the test.
                                cmd.args([format!("-{name}"), value]);
                            }
                            Opt::Long(name, value) => {
                                cmd.args([format!("--{name}"), value]);
                            }
                        }
                    }

                    for arg in &inv.args {
                        cmd.arg(arg.to_string());
                    }

                    if let Some(stdin) = inv.stdin {
                        cmd.write_stdin(stdin);
                    }

                    Ok(cmd)
                }
            }
        }
    }

    /// Multiple commands can be piped together.
    #[derive(Debug, Clone, PartialEq, Eq)]
    struct PipedPrograms {
        /// The program forming the pipe.
        programs: VecDeque<Program>,
        /// The expected outcome of the *entire* pipe. Any failure anywhere in the pipe
        /// should cause overall failure (like `pipefail`).
        should_fail: bool,
        /// Is this not actually a pipe, but a single, standalone program?
        ///
        /// Checking for the length of `programs` might not suffice, as programs are
        /// dropped from it as it's processed.
        standalone: bool,
    }

    impl PipedPrograms {
        /// Assembles a list of programs into a pipe.
        ///
        /// The first program is specially treated, and needs to be able to produce some
        /// stdin to kick things off (unless there is only one `standalone` program in
        /// the pipe). The passed `stdout` is the expected output of the last program,
        /// aka the entire pipe. All programs in the middle (after *second*, before
        /// last), which can be an unbounded number, are later on fed their stdin
        /// dynamically, from the previous program's output, with their stdout becoming
        /// the next piped program's stdin, and so on. These stdins and stdouts are not
        /// set here.
        ///
        /// [`Program::Cat`] invocations may need access to [`Snippets`], so those are
        /// provided as well; pipes may also be *expected* to fail.
        ///
        /// For example, a chain might look like:
        ///
        /// ```text
        /// echo 'hello' | butest -a | butest -b | butest --cee | butest -d
        /// ^^^^^^^^^^^^   ^^^^^^^^^
        ///  produces      is fed      ^^^^^^^^^^^^^^^^^^^^^^^^   ^^^^^^^^^
        ///  initial       stdin to      these are provided       last one
        ///  stdin         kick          their stdin *later*      in the
        ///                things        on, and produce their    pipe:
        ///                off           stdout dynamically too   its
        ///                                                       stdout is
        ///                                                       set here,
        ///                                                       its stdin
        ///                                                       provided
        ///                                                       dynamically
        /// ```
        ///
        /// with some binary under test (`butest`).
        fn assemble(
            chain: impl Iterator<Item = Program>,
            stdout: Option<String>,
            snippets: &Snippets,
            should_fail: bool,
        ) -> Result<Self, &'static str> {
            let mut programs = chain.collect::<VecDeque<_>>();
            eprintln!("Will assemble programs: {programs:?}");

            // There's a trailing newline inserted which ruins/fails diffs.
            let stdout = stdout.map(|s| s.trim_end().to_string());

            let mut standalone = false;

            let first = programs
                .pop_front()
                .ok_or("Should have at least one program in pipe")?;

            let stdin = match &first {
                Program::Echo(inv) => Some(
                    inv.args
                        .first()
                        .ok_or("Echo should have an argument")?
                        .to_string(),
                ),
                Program::Cat(inv) => {
                    let file_name = inv.args.first().ok_or("Cat should have an argument")?;

                    Some(
                        snippets
                            .get(&file_name.0)
                            .ok_or("Snippet should be present")?
                            .original
                            .clone()
                            .ok_or("Snippet should have an original")?,
                    )
                }
                Program::Self_(..) => {
                    standalone = true;
                    None
                }
            };

            // Set the *second*, if any, command's standard input.
            match programs.front_mut() {
                Some(Program::Echo(_) | Program::Cat(_)) => {
                    return Err("Stdin-generating program should not be in the middle of a pipe")
                }
                Some(Program::Self_(inv)) => {
                    inv.stdin = stdin;
                }
                None => {
                    // Nothing to do; assert flag was set already.
                    assert!(standalone);
                }
            }

            // Set the expected standard output of the *entire* pipe, aka the last
            // program's standard output.
            let mut first = first;
            match programs.back_mut() {
                Some(Program::Echo(_) | Program::Cat(_)) => {
                    return Err("Stdin-generating program should not be at the end of a pipe")
                }
                Some(Program::Self_(inv)) => {
                    inv.stdout = if should_fail {
                        // No stdout needed if command fails anyway
                        None
                    } else if let Program::Cat(inv) = first {
                        Some(
                            snippets
                                .get(&inv.args.first().expect("Cat should have an argument").0)
                                .expect("Cat invocation needs a snippet")
                                .output
                                .clone()
                                .unwrap_or_else(|| {
                                    stdout.expect(
                                        "Snippet for cat has no output, so stdout is required",
                                    )
                                }),
                        )
                    } else {
                        Some(stdout.expect("Stdout should be given for non-`cat`-fed program"))
                    }
                }
                None => {
                    match &mut first {
                        // There is no 'last program': we have a stand-alone one.
                        Program::Echo(_) | Program::Cat(_) => {
                            return Err("Illegal standalone program")
                        }
                        Program::Self_(inv) => {
                            inv.stdout = Some(
                                stdout.expect("Stdout should be given for standalone program"),
                            );
                        }
                    };

                    assert!(standalone, "Should have been set before.");

                    // Put it back!
                    programs.push_back(first);
                }
            }

            assert!(!programs.is_empty());
            Ok(Self {
                programs,
                should_fail,
                standalone,
            })
        }
    }

    impl IntoIterator for PipedPrograms {
        type Item = Program;
        type IntoIter = std::collections::vec_deque::IntoIter<Self::Item>;

        fn into_iter(self) -> Self::IntoIter {
            self.programs.into_iter()
        }
    }

    /// Parses a code block such as:
    ///
    /// ```console
    /// $ echo 'some input' | program --flag arg1 arg2 | program2 arg1 # Optional comment
    /// some output
    /// ```
    ///
    /// into a proper [`CommandUnderTest`], such that `program` can be run with those
    /// found arguments. Note this routine's job is *not* to deal with the backticks or
    /// the `console` language tag, but rather to parse the command and its output (so
    /// anything in between). If applied multiple times, blocks such as these can be
    /// detected:
    ///
    /// ```console
    /// $ echo 'some input' | program --flag arg1 arg2  # Optional comment
    /// some output
    /// $ echo 'some other input' | program arg1
    /// some other output
    /// ```
    fn parse_piped_programs_with_prompt_and_output<'a>(
        input: &'a str,
        snippets: &Snippets,
    ) -> IResult<&'a str, PipedPrograms> {
        let prompt = '$';
        let (input, _) = opt(char(prompt))(input)?;
        let (input, _) = space0(input)?;

        let (input, programs) = parse_piped_programs(input)?;
        eprintln!("Parsed programs: {programs:#?}");

        // Advance to end; this eats optional comments and trailing whitespace.
        let (input, tail) = take_until("\n")(input)?;
        let should_fail = tail.contains("will fail");
        let (input, _) = line_ending(input)?;

        // Parse stdout; anything up to the next prompt. Be careful about `$` signs
        // *inside* of expected stdout: only `\n$` is a new prompt.
        let (input, (stdout_chars, start_of_next_input)) =
            many_till(anychar, alt((preceded(line_ending, tag("$")), eof)))(input)?;

        assert!(
            start_of_next_input == "$" || start_of_next_input.is_empty(),
            "Unexpected parsing result: {start_of_next_input} (remaining input: {})",
            input.escape_debug()
        );

        let stdout = if stdout_chars.is_empty() {
            None
        } else {
            Some(stdout_chars.into_iter().join(""))
        };

        eprintln!("Remaining input: {input:?}");
        eprintln!("Parsed stdout: {stdout:?}");

        Ok((
            input,
            PipedPrograms::assemble(programs.into_iter(), stdout, snippets, should_fail)
                .expect("Should be able to assemble"),
        ))
    }

    /// <https://docs.rs/nom/7.1.3/nom/recipes/index.html#wrapper-combinators-that-eat-whitespace-before-and-after-a-parser>
    /// A combinator that takes a parser `inner` and produces a parser that also consumes both leading and
    /// trailing whitespace, returning the output of `inner`.
    fn maybe_ws<'a, F, O, E>(inner: F) -> impl FnMut(&'a str) -> IResult<&'a str, O, E>
    where
        F: 'a + Fn(&'a str) -> IResult<&'a str, O, E>,
        E: ParseError<&'a str>,
    {
        delimited(space0, inner, space0)
    }

    /// Parses a single, whole program invocation.
    fn parse_program(input: &str) -> IResult<&str, Program> {
        // Interior mutability is fine, as the different closures aliasing this run
        // sequentially, never at once (is using this and `map` of `nom` an
        // anti-pattern? works quite well...)
        let inv = Rc::new(RefCell::new(Invocation::default()));

        let (input, (name, _flags, _args)) = tuple((
            maybe_ws(ascii_alpha1),
            many0(alt((
                map(
                    tuple((
                        preceded(
                            // Long options. Hard-coded, as otherwise it's undecidable
                            // whether an option is supposed to have a value or not
                            // (just a flag). Alternatively, import `clap::Cli` here and
                            // `try_get_matches` with it, but cannot/don't want to
                            // expose (`pub`) that.
                            //
                            // Failure to add values here will result in bizarre and
                            // wrong order of arguments. For example, if `--some-option
                            // some_value` is given and `--some-option` is not
                            // registered here, `--some-option` will be understood as a
                            // *flag*, and `some_value` as a *positional argument*.
                            tag("--"),
                            alt((
                                // Careful: all `--lang-query` options need to come
                                // first; otherwise, the `--lang` options eat them and
                                // results turn bad (complaining that `-query` is not a
                                // valid value). Parsing is brittle here :-(
                                tag("csharp-query"),
                                tag("go-query"),
                                tag("hcl-query"),
                                tag("python-query"),
                                tag("rust-query"),
                                tag("typescript-query"),
                                //
                                tag("csharp"),
                                tag("glob"),
                                tag("go"),
                                tag("hcl"),
                                tag("python"),
                                tag("rust"),
                                tag("stdin-override-to"),
                                tag("threads"),
                                tag("typescript"),
                                //
                                // Shorthands. NOTE: only a limited number of elements
                                // can go here. `nom` is generic, and this tuple inside
                                // `alt` is limited to 21 members.
                                tag("cs"),
                                tag("py"),
                                tag("rs"),
                                tag("ts"),
                            )),
                        ),
                        cut(
                            // `cut`: should we get here, and not succeed, parsing has
                            // to fail entirely. Else we continue with bad data.
                            delimited(
                                space1,
                                // Quoting always is technically overkill, but much
                                // simpler and safer
                                parse_quoted,
                                space0,
                            ),
                        ),
                    )),
                    |findings| {
                        inv.borrow_mut().opts.push(findings.into());
                        findings
                    },
                ),
                map(
                    tuple((
                        preceded(
                            // Long flags, like `--long-flag`. No values. Can contain
                            // hyphens itself.
                            tag("--"),
                            take_while1(|c: char| c == '-' || c.is_ascii_alphanumeric()),
                        ),
                        space0,
                    )),
                    |findings: (&str, &str)| {
                        let (flag, _space) = findings;
                        inv.borrow_mut().flags.push(flag.into());
                        findings
                    },
                ),
                map(
                    tuple((
                        // Short flags, like `-s`, but also `-sGu`. No values.
                        preceded(char('-'), ascii_alphanumeric1),
                        space0,
                    )),
                    |found: (&str, &str)| {
                        let (flag, _space) = found;

                        flag.chars()
                            .for_each(|c| inv.borrow_mut().flags.push(c.into()));

                        found
                    },
                ),
            ))),
            many0(alt((
                map(
                    // Regular, quoted positional args
                    maybe_ws(parse_quoted),
                    |s: &str| {
                        inv.borrow_mut().args.push(s.into());

                        // Owned because type needs to align with other list members
                        s.to_owned()
                    },
                ),
                map(
                    // There's also file names, which cannot occur quoted in shell
                    // contexts
                    tuple((ascii_alphanumeric1, char('.'), ascii_alphanumeric1)),
                    |parts: (&str, char, &str)| {
                        let (stem, sep, suffix) = parts;
                        let file_name = format!("{stem}{sep}{suffix}");

                        inv.borrow_mut().args.push(file_name.as_str().into());

                        file_name
                    },
                ),
            ))),
        ))(input)?;

        let (input, _) = space0(input)?;

        let program = Program::from_name(name, inv.borrow().clone());
        Ok((input, program))
    }

    fn parse_piped_programs(input: &str) -> IResult<&str, Vec<Program>> {
        separated_list1(tag("|"), parse_program)(input)
    }

    /// Parses multiple pairs of 'command and output' into a list of them.
    fn parse_code_blocks<'a>(
        input: &'a str,
        snippets: &Snippets,
    ) -> IResult<&'a str, Vec<PipedPrograms>> {
        many1(|input| parse_piped_programs_with_prompt_and_output(input, snippets))(input)
    }

    /// <https://stackoverflow.com/a/58907488/11477374>
    fn parse_quoted(input: &str) -> IResult<&str, &str> {
        let esc = escaped(none_of(r"'"), '\\', tag("'"));
        let esc_or_empty = alt((esc, tag("")));
        let res = delimited(tag("'"), esc_or_empty, tag("'"))(input)?;

        Ok(res)
    }

    /// Supported Operating systems.
    #[derive(Debug, Clone, PartialEq, Eq)]
    enum OS {
        Linux,
        #[allow(clippy::enum_variant_names)]
        MacOS,
        Windows,
    }

    impl TryFrom<String> for OS {
        type Error = &'static str;

        /// Convert from a value as returned by [`std::env::consts::OS`].
        fn try_from(value: String) -> Result<Self, Self::Error> {
            match value.as_str() {
                "linux" => Ok(Self::Linux),
                "macos" => Ok(Self::MacOS),
                "windows" => Ok(Self::Windows),
                // Just a double check to ensure parsing of options went right.
                _ => Err("unknown OS"),
            }
        }
    }

    #[derive(Debug, Clone, PartialEq, Eq, Default)]
    struct BlockOptions {
        /// The file name to associate with this code block, if any.
        filename: Option<String>,
        /// Skip when running on any of these operating systems.
        skip_os: Option<Vec<OS>>,
    }

    impl<S> From<S> for BlockOptions
    where
        S: AsRef<str>,
    {
        fn from(value: S) -> Self {
            let string = value.as_ref();
            let mut options: HashMap<&str, String> = string
                .split(';')
                .map(|pair| {
                    pair.split_once('=')
                        .unwrap_or_else(|| panic!("need a value for key-value pair: {}", pair))
                })
                .map(|(k, v)| (k, v.to_string()))
                .collect();

            let get_many = |s: String| s.split(',').map(ToOwned::to_owned).collect_vec();

            let res = Self {
                filename: options.remove("file"),
                skip_os: options.remove("skip-os").map(get_many).map(|oss| {
                    oss.into_iter()
                        .map(|os| os.try_into().expect("any used OS to be known"))
                        .collect_vec()
                }),
            };

            assert!(options.is_empty(), "unknown keys in options: {options:?}");

            res
        }
    }

    #[derive(Debug, Clone, PartialEq, Eq, Default)]
    struct Snippet {
        original: Option<String>,
        output: Option<String>,
    }

    impl Snippet {
        fn join(self, other: Self) -> Self {
            Self {
                original: Some(
                    self.original
                        .or(other.original)
                        .expect("After joining, snippet should have an original"),
                ),
                output: Some(
                    self.output
                        .or(other.output)
                        .expect("After joining, snippet should have an output"),
                ),
            }
        }
    }

    type Snippets = HashMap<String, Snippet>;

    fn get_readme_snippets() -> Snippets {
        let mut snippets = HashMap::new();

        map_on_markdown_codeblocks(DOCUMENT, |ncb| {
            if let Some((_language, options)) = ncb.info.split_once(' ') {
                let mut snippet = Snippet::default();
                let options: BlockOptions = options.into();

                if let BlockOptions {
                    filename: Some(mut file_name),
                    ..
                } = options
                {
                    file_name = if let Some(stripped_file_name) = file_name.strip_prefix("output-")
                    {
                        snippet.output = Some(ncb.literal.trim_end().to_owned());
                        stripped_file_name.to_owned()
                    } else {
                        snippet.original = Some(ncb.literal.trim_end().to_owned());
                        file_name
                    };

                    if let Some(other) = snippets.remove(&file_name) {
                        snippet = snippet.join(other);
                    }

                    snippets.insert(file_name, snippet);
                }
            }
        });

        eprintln!("Snippets: {snippets:#?}");

        snippets
    }

    fn get_readme_program_pipes(snippets: &Snippets) -> Vec<PipedPrograms> {
        let mut pipes = Vec::new();

        map_on_markdown_codeblocks(DOCUMENT, |ncb| {
            let (language, options): (&str, Option<BlockOptions>) = ncb
                .info
                .split_once(' ')
                .map_or_else(|| (ncb.info.as_str(), None), |(l, o)| (l, Some(o.into())));

            if let Some(BlockOptions {
                skip_os: Some(oss), ..
            }) = options
            {
                let curr_os: OS = std::env::consts::OS
                    .to_owned()
                    .try_into()
                    .expect("any used OS to be known");
                if oss.contains(&curr_os) {
                    eprintln!("Skipping OS: {curr_os:?}");
                    return;
                }
            }

            if language == "console" || language == "bash" {
                let (_, commands) = parse_code_blocks(&ncb.literal, &snippets.clone())
                    .finish()
                    .expect("Anything in `console` should be parseable as a command");

                pipes.extend(commands);
            }
        });

        eprintln!("Piped programs: {pipes:?}");

        pipes
    }

    fn map_on_markdown_codeblocks(markdown: &str, mut f: impl FnMut(NodeCodeBlock)) {
        let arena = Arena::new();

        let root = parse_document(&arena, markdown, &ComrakOptions::default());

        root.descendants().for_each(|node| {
            let value = node.to_owned().data.borrow().value.clone();

            if let NodeValue::CodeBlock(ncb) = value {
                f(ncb);
            }
        });
    }

    #[test]
    fn test_readme_code_blocks() {
        let snippets = get_readme_snippets();
        let pipes = get_readme_program_pipes(&snippets);

        for pipe in pipes {
            let mut previous_stdin = None;
            let should_fail = pipe.should_fail;
            let standalone = pipe.standalone;
            for mut program in pipe {
                if standalone {
                    program.force_ignore_stdin();
                }
                let program = program; // de-mut

                let mut cmd = Command::try_from(program.clone())
                    .expect("Should be able to convert invocation to cmd to run");

                // We're testing and need determinism. This hard-codes a flag!
                cmd.arg("--sorted");

                if let Some(previous_stdin) = previous_stdin {
                    cmd.write_stdin(previous_stdin);
                }

                eprintln!("Running command: {cmd:?}");

                let mut assertion = cmd.assert();

                assertion = if should_fail {
                    assertion.failure()
                } else {
                    assertion.success()
                };

                let mut observed_stdout = String::from_utf8(assertion.get_output().stdout.clone())
                    .expect("Stdout should be given as UTF-8");
                observed_stdout = {
                    let s = observed_stdout.trim_end();
                    s.to_owned()
                };

                if let Some(expected_stdout) = program.stdout().clone() {
                    if cfg!(target_os = "windows") {
                        observed_stdout = fix_windows_output(observed_stdout);
                    }

                    if observed_stdout != expected_stdout {
                        // Write to files for easier inspection
                        let (mut obs_f, mut exp_f) = (
                            ManuallyDrop::new(NamedTempFile::new().unwrap()),
                            ManuallyDrop::new(NamedTempFile::new().unwrap()),
                        );

                        obs_f.write_all(observed_stdout.as_bytes()).unwrap();
                        exp_f.write_all(expected_stdout.as_bytes()).unwrap();

                        // Now panic as usual, for the usual output
                        assert_eq!(
                            // Get some more readable output diff compared to
                            // `assert::Command`'s `stdout()` function, for which diffing
                            // whitespace is very hard.
                            expected_stdout,
                            observed_stdout,
                            "Output differs; for inspection see observed stdout at '{}', expected stdout at '{}'",
                            obs_f.path().display(),
                            exp_f.path().display()
                        );

                        // Temporary files remain, they're not dropped.

                        unreachable!();
                    }
                }

                // Pipe stdout to stdin of next run...
                previous_stdin = Some(observed_stdout);
            }
        }
    }

    fn fix_windows_output(mut input: String) -> String {
        input = unixfy_file_paths(&input);
        input = remove_exe_suffix(&input);

        input
    }

    fn remove_exe_suffix(input: &str) -> String {
        input.replace(
            concat!(env!("CARGO_PKG_NAME"), ".exe"),
            env!("CARGO_PKG_NAME"),
        )
    }

    /// The document under test might contain hard-coded Unix file paths. When running
    /// under Windows, where `\` might be printed as the path separator, tests will
    /// break. So hack strings which look like paths to spell `/` instead of `\` as
    /// their path separator.
    fn unixfy_file_paths(input: &str) -> String {
        // Pattern for Windows-style file paths:
        //
        // ```text
        // No Match: /usr/bin/local
        // No Match: bin/local
        // Matches: test\files
        // Matches: test\some\more\files
        // Matches: test
        // Matches: test\some\file.py
        //```
        //
        // https://regex101.com/r/NURPe2/1
        let pattern = Regex::new(r"^([a-z]+\\)*[a-z]+(\.[a-z]+)?$").unwrap();
        let mut res = input
            .lines()
            .map(|s| {
                if pattern.is_match(s).unwrap() {
                    let res = s.replace('\\', "/");
                    eprintln!("Replaced Windows path-like string: {s} -> {res}");
                    res
                } else {
                    s.to_owned()
                }
            })
            .collect::<Vec<String>>()
            .join("\n");

        if input.ends_with('\n') {
            res.push('\n');
        }

        res
    }
}
