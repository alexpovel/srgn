#[cfg(all(test, feature = "all"))]
mod tests {
    use assert_cmd::Command;
    use comrak::{
        nodes::{NodeCodeBlock, NodeValue},
        parse_document, Arena, ComrakOptions,
    };
    use core::fmt;
    use nom::{
        branch::alt,
        bytes::complete::{escaped, is_not, tag, take_until, take_while1},
        character::complete::{
            alpha1 as ascii_alpha1, alphanumeric1 as ascii_alphanumeric1, char, line_ending,
            none_of, space0, space1,
        },
        combinator::{cut, map, opt},
        error::ParseError,
        multi::{many0, many1, separated_list1},
        sequence::{delimited, preceded, tuple},
        Finish, IResult,
    };
    use std::{
        cell::RefCell,
        collections::{HashMap, VecDeque},
        rc::Rc,
    };
    use unescape::unescape;

    const PROGRAM_NAME: &str = env!("CARGO_PKG_NAME");

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
                Self::Short(c) => write!(f, "-{}", c),
                Self::Long(s) => write!(f, "--{}", s),
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

        fn name(&self) -> &str {
            match self {
                Self::Echo(_) => "echo",
                Self::Cat(_) => "cat",
                Self::Self_(_) => PROGRAM_NAME,
            }
        }

        fn stdout(&self) -> Option<String> {
            match self {
                Self::Echo(inv) => inv.stdout.clone(),
                Self::Cat(inv) => inv.stdout.clone(),
                Self::Self_(inv) => inv.stdout.clone(),
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
                    let mut cmd = Command::cargo_bin(name).expect("Should be able to find binary");

                    for flag in inv.flags {
                        cmd.arg(flag.to_string());
                    }

                    for arg in inv.args {
                        cmd.arg(arg.to_string());
                    }

                    for opt in inv.opts {
                        match opt {
                            Opt::Short(name, value) => {
                                // Push these separately, as `arg` will escape the
                                // value, and something like `--option value` will be
                                // taken as a single arg, breaking the test.
                                cmd.arg(format!("-{name}"));
                                cmd.arg(value);
                            }
                            Opt::Long(name, value) => {
                                cmd.arg(format!("--{name}"));
                                cmd.arg(value);
                            }
                        }
                    }

                    // Empty string will be overwritten later on anyway. This saves a bunch
                    // of code later.
                    cmd.write_stdin(inv.stdin.unwrap_or_default());

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
    }

    impl PipedPrograms {
        /// Assembles a list of programs into a pipe.
        ///
        /// The first program is specially treated, and needs to be able to produce some
        /// stdin. The passed `stdout` is the expected output of the last program, aka
        /// the entire pipe.
        fn assemble(
            chain: impl Iterator<Item = Program>,
            stdout: Option<&str>,
            snippets: Snippets,
            should_fail: bool,
        ) -> Result<Self, &str> {
            let mut programs = chain.collect::<VecDeque<_>>();

            let first = programs
                .pop_front()
                .ok_or("Should have at least one program in pipe")?;

            let stdin = match &first {
                Program::Echo(inv) => inv
                    .args
                    .first()
                    .ok_or("Echo should have an argument")?
                    .to_string(),
                Program::Cat(inv) => {
                    let file_name = inv.args.first().ok_or("Cat should have an argument")?;

                    snippets
                        .get(&file_name.0)
                        .ok_or("Snippet should be present")?
                        .original
                        .clone()
                        .ok_or("Snippet should have an original")?
                }
                _ => return Err("First command should be able to produce stdin."),
            };

            match programs
                .front_mut()
                .expect("No second program to assemble with")
            {
                Program::Echo(_) | Program::Cat(_) => {
                    return Err("Stdin-generating program should not be in the middle of a pipe")
                }
                Program::Self_(inv) => {
                    inv.stdin = Some(stdin);
                }
            }

            match programs
                .back_mut()
                .expect("No last program to assemble with")
            {
                Program::Echo(_) | Program::Cat(_) => {
                    return Err("Stdin-generating program should not be at the end of a pipe")
                }
                Program::Self_(inv) => {
                    inv.stdout = if should_fail {
                        // No stdout needed if command fails anyway
                        None
                    } else if let Program::Cat(inv) = first {
                        assert!(
                            stdout.is_none(),
                            "Cat output should be given as extra snippet, not inline"
                        );

                        Some(
                            snippets
                                .get(&inv.args.first().expect("Cat should have an argument").0)
                                .ok_or("Snippet should be present")?
                                .output
                                .clone()
                                .expect("Snippet should have an output"),
                        )
                    } else {
                        Some(
                            stdout
                                .expect("Stdout should be given for non-`cat`-fed program")
                                // No patience for hard-to-diff, fiddly, invisible
                                // whitespace issues, even though it would be "more
                                // correct"
                                .trim_end()
                                .to_string(),
                        )
                    }
                }
            }

            Ok(Self {
                programs,
                should_fail,
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
    fn parse_piped_programs_with_prompt_and_output(
        input: &str,
        snippets: Snippets,
    ) -> IResult<&str, PipedPrograms> {
        let prompt = '$';
        let (input, _) = opt(char(prompt))(input)?;
        let (input, _) = space0(input)?;

        let (input, programs) = parse_piped_programs(input)?;
        eprintln!("Parsed programs: {:#?}", programs);

        // Advance to end; this eats optional comments and trailing whitespace.
        let (input, tail) = take_until("\n")(input)?;
        let should_fail = tail.contains("will fail");
        let (input, _) = line_ending(input)?;

        // Parse stdout; anything up to the next prompt.
        let (input, stdout) = opt(is_not(prompt.to_string().as_str()))(input)?;
        eprintln!("Parsed stdout: {:#?}", stdout);

        Ok((
            input,
            PipedPrograms::assemble(programs.into_iter(), stdout, snippets, should_fail)
                .expect("Should be able to assemble"),
        ))
    }

    /// https://docs.rs/nom/7.1.3/nom/recipes/index.html#wrapper-combinators-that-eat-whitespace-before-and-after-a-parser
    /// A combinator that takes a parser `inner` and produces a parser that also consumes both leading and
    /// trailing whitespace, returning the output of `inner`.
    fn maybe_ws<'a, F: 'a, O, E: ParseError<&'a str>>(
        inner: F,
    ) -> impl FnMut(&'a str) -> IResult<&'a str, O, E>
    where
        F: Fn(&'a str) -> IResult<&'a str, O, E>,
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
                            tag("--"),
                            alt((
                                tag("csharp-query"),
                                tag("csharp"),
                                tag("go-query"),
                                tag("go"),
                                tag("python-query"),
                                tag("python"),
                                tag("typescript-query"),
                                tag("typescript"),
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
                                space1,
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
                        let file_name = format!("{}{}{}", stem, sep, suffix);

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
    fn parse_code_blocks(input: &str, snippets: Snippets) -> IResult<&str, Vec<PipedPrograms>> {
        many1(|input| parse_piped_programs_with_prompt_and_output(input, snippets.clone()))(input)
    }

    /// https://stackoverflow.com/a/58907488/11477374
    fn parse_quoted(input: &str) -> IResult<&str, &str> {
        let esc = escaped(none_of(r"'"), '\\', tag("'"));
        let esc_or_empty = alt((esc, tag("")));
        let res = delimited(tag("'"), esc_or_empty, tag("'"))(input)?;

        Ok(res)
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

        map_on_markdown_codeblocks(include_str!("../README.md"), |ncb| {
            if let Some((_language, mut file_name)) = ncb.info.split_once(' ') {
                let mut snippet = Snippet::default();

                file_name = match file_name.strip_prefix("output-") {
                    Some(stripped_file_name) => {
                        snippet.output = Some(ncb.literal);
                        stripped_file_name
                    }
                    None => {
                        snippet.original = Some(ncb.literal);
                        file_name
                    }
                };

                if let Some((_, other)) = snippets.remove_entry(file_name) {
                    snippet = snippet.join(other);
                }

                snippets.insert(file_name.to_owned(), snippet);
            }
        });

        eprintln!("Snippets: {:#?}", snippets);

        snippets
    }

    fn get_readme_program_pipes(snippets: Snippets) -> Vec<PipedPrograms> {
        let mut pipes = Vec::new();

        map_on_markdown_codeblocks(include_str!("../README.md"), |ncb| {
            if ncb.info == "console" || ncb.info == "bash" {
                let (_, commands) = parse_code_blocks(&ncb.literal, snippets.clone())
                    .finish()
                    .expect("Anything in `console` should be parseable as a command");

                pipes.extend(commands);
            }
        });

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
        let pipes = get_readme_program_pipes(snippets);

        for pipe in pipes {
            let mut previous_stdin = None;
            let should_fail = pipe.should_fail;
            for program in pipe {
                let mut cmd = Command::try_from(program.clone())
                    .expect("Should be able to convert invocation to cmd to run");

                if let Some(previous_stdin) = previous_stdin {
                    cmd.write_stdin(previous_stdin);
                }

                eprintln!("Running command: {:?}", cmd);

                let mut assertion = cmd.assert();

                assertion = if should_fail {
                    assertion.failure()
                } else {
                    assertion.success()
                };

                if let Some(stdout) = program.stdout().clone() {
                    assertion.stdout(stdout);
                }

                // Pipe stdout to stdin of next run...
                previous_stdin = Some(
                    String::from_utf8(cmd.assert().get_output().stdout.clone())
                        .expect("Stdout should be given as UTF-8"),
                );
            }
        }
    }
}
