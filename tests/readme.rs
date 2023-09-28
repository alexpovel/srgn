#[cfg(test)]
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
            none_of, space0,
        },
        combinator::map,
        error::ParseError,
        multi::{many0, many1, separated_list1},
        sequence::{delimited, tuple},
        Finish, IResult,
    };
    use std::{cell::RefCell, collections::VecDeque, rc::Rc};
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
        args: Vec<Arg>,
        //
        stdin: Option<String>,
        stdout: Option<String>,
    }

    #[derive(Debug, Clone, PartialEq, Eq)]
    enum Program {
        /// The `echo` program, used to generate stdin for the program under test.
        Echo(Invocation),
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
                PROGRAM_NAME => Self::Self_(invocation),
                _ => panic!("Unsupported program name: {}", name),
            }
        }

        fn name(&self) -> &str {
            match self {
                Self::Echo(_) => "echo",
                Self::Self_(_) => PROGRAM_NAME,
            }
        }

        fn stdout(&self) -> Option<String> {
            match self {
                Self::Echo(inv) => inv.stdout.clone(),
                Self::Self_(inv) => inv.stdout.clone(),
            }
        }
    }

    impl TryFrom<Program> for Command {
        type Error = &'static str;

        fn try_from(prog: Program) -> Result<Self, Self::Error> {
            let name = prog.name().to_string();

            match prog {
                Program::Echo(_) => Err("Echo cannot be run, only used to generate stdin"),
                Program::Self_(inv) => {
                    let mut cmd = Command::cargo_bin(name).expect("Should be able to find binary");

                    for flag in inv.flags {
                        cmd.arg(flag.to_string());
                    }

                    for arg in inv.args {
                        cmd.arg(arg.to_string());
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
    struct PipedPrograms(VecDeque<Program>);

    impl PipedPrograms {
        /// Assembles a list of programs into a pipe.
        ///
        /// The first program is specially treated, and needs to be able to produce some
        /// stdin. The passed `stdout` is the expected output of the last program, aka
        /// the entire pipe.
        fn assemble(chain: impl Iterator<Item = Program>, stdout: &str) -> Result<Self, &str> {
            let mut chain = chain.collect::<VecDeque<_>>();

            let stdin = match chain.pop_front() {
                Some(p) => match p {
                    Program::Echo(mut inv) => {
                        inv.args.pop().ok_or("Echo should have an argument")?
                    }
                    _ => return Err("First command should be able to produce stdin."),
                },
                None => {
                    return Err("Should have at least one program in pipe");
                }
            };

            match chain
                .front_mut()
                .expect("No second program to assemble with")
            {
                Program::Echo(_) => return Err("Echo should not be in the middle of a pipe"),
                Program::Self_(inv) => {
                    inv.stdin = Some(stdin.to_string());
                }
            }

            match chain.back_mut().expect("No last program to assemble with") {
                Program::Echo(_) => return Err("Echo should not be at the end of a pipe"),
                Program::Self_(inv) => {
                    inv.stdout = Some(stdout.into());
                }
            }

            Ok(Self(chain))
        }
    }

    impl IntoIterator for PipedPrograms {
        type Item = Program;
        type IntoIter = std::collections::vec_deque::IntoIter<Self::Item>;

        fn into_iter(self) -> Self::IntoIter {
            self.0.into_iter()
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
    fn parse_piped_programs_with_prompt_and_output(input: &str) -> IResult<&str, PipedPrograms> {
        let prompt = '$';
        let (input, _) = char(prompt)(input)?;
        let (input, _) = space0(input)?;

        let (input, programs) = parse_piped_programs(input)?;
        eprintln!("Parsed programs: {:#?}", programs);

        // Advance to end; this eats optional comments and trailing whitespace.
        let (input, _) = take_until("\n")(input)?;
        let (input, _) = line_ending(input)?;

        // Parse stdout; anything up to the next prompt.
        let (input, stdout) = is_not(prompt.to_string().as_str())(input)?;
        eprintln!("Parsed stdout: {:#?}", stdout);

        let stdout = stdout.trim_end(); // Removes flakiness and hard-to-diff stuff

        Ok((
            input,
            PipedPrograms::assemble(programs.into_iter(), stdout)
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
                    delimited(
                        // Long flags, like `--long-flag`. No values. Can contain
                        // hyphens itself.
                        tag("--"),
                        take_while1(|c: char| c == '-' || c.is_ascii_alphanumeric()),
                        space0,
                    ),
                    |s: &str| {
                        inv.borrow_mut().flags.push(s.into());
                        s
                    },
                ),
                map(
                    // Short flags, like `-s`, but also `-sGu`. No values.
                    delimited(char('-'), ascii_alphanumeric1, space0),
                    |s: &str| {
                        s.chars()
                            .for_each(|c| inv.borrow_mut().flags.push(c.into()));
                        s
                    },
                ),
            ))),
            many0(alt((map(
                // Quoted, positional arguments
                maybe_ws(parse_quoted),
                |s: &str| {
                    inv.borrow_mut().args.push(s.into());
                    s
                },
            ),))),
        ))(input)?;

        let (input, _) = space0(input)?;

        let program = Program::from_name(name, inv.borrow().clone());
        Ok((input, program))
    }

    fn parse_piped_programs(input: &str) -> IResult<&str, Vec<Program>> {
        separated_list1(tag("|"), parse_program)(input)
    }

    /// Parses multiple pairs of 'command and output' into a list of them.
    fn parse_code_blocks(input: &str) -> IResult<&str, Vec<PipedPrograms>> {
        many1(parse_piped_programs_with_prompt_and_output)(input)
    }

    /// https://stackoverflow.com/a/58907488/11477374
    fn parse_quoted(input: &str) -> IResult<&str, &str> {
        let esc = escaped(none_of(r"'"), '\\', tag("'"));
        let esc_or_empty = alt((esc, tag("")));
        let res = delimited(tag("'"), esc_or_empty, tag("'"))(input)?;

        Ok(res)
    }

    fn get_all_commands_under_test_pipes_from_readme() -> Vec<PipedPrograms> {
        let arena = Arena::new();

        let root = parse_document(
            &arena,
            include_str!("../README.md"),
            &ComrakOptions::default(),
        );

        let mut pipes = Vec::new();
        let console = String::from("console");

        root.descendants().for_each(|node| {
            let value = node.to_owned().data.borrow().value.clone();

            if let NodeValue::CodeBlock(NodeCodeBlock { info, literal, .. }) = value {
                if info == console {
                    let (_, commands) = parse_code_blocks(&literal)
                        .finish()
                        .expect("Anything in `console` should be parseable as a command");

                    pipes.extend(commands);
                }
            }
        });

        pipes
    }

    #[test]
    fn test_readme_code_blocks() {
        let pipes = get_all_commands_under_test_pipes_from_readme();

        for pipe in pipes {
            let mut previous_stdin = None;
            for program in pipe {
                let mut cmd = Command::try_from(program.clone())
                    .expect("Should be able to convert invocation to cmd to run");

                if let Some(previous_stdin) = previous_stdin {
                    cmd.write_stdin(previous_stdin);
                }

                eprintln!("Running command: {:?}", cmd);

                if let Some(stdout) = program.stdout().clone() {
                    // `success` takes ownership so can't test separately.
                    cmd.assert().success().stdout(stdout);
                } else {
                    cmd.assert().success();
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
