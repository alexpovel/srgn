#[cfg(test)]
mod tests {
    use core::fmt;
    use std::{cell::RefCell, collections::VecDeque, rc::Rc};

    use assert_cmd::Command;
    use comrak::{
        nodes::{NodeCodeBlock, NodeValue},
        parse_document, Arena, ComrakOptions,
    };

    use nom::{
        branch::alt,
        bytes::complete::{escaped, tag, take_till, take_until, take_while1},
        character::{
            complete::{
                alpha1 as ascii_alpha1, alphanumeric1 as ascii_alphanumeric1, char, line_ending,
                multispace0, multispace1, none_of, space0, space1,
            },
            is_alphanumeric,
        },
        combinator::{map, opt, recognize},
        error::ParseError,
        multi::{many0, many1, separated_list1},
        sequence::{delimited, preceded, tuple, Tuple},
        Finish, IResult,
    };
    use shell_words::split;
    use unescape::unescape;

    const PROGRAM_NAME: &str = env!("CARGO_PKG_NAME");

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

    impl From<String> for Flag {
        fn from(s: String) -> Self {
            Self::Long(s)
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

    #[derive(Debug, Clone, PartialEq, Eq)]
    struct Option_ {
        name: Flag,
        value: String,
    }

    #[derive(Debug, Clone, PartialEq, Eq)]
    struct Arg(String);

    impl From<&str> for Arg {
        fn from(s: &str) -> Self {
            Self(s.to_string())
        }
    }

    impl From<Arg> for String {
        fn from(a: Arg) -> Self {
            a.0
        }
    }

    impl<'a> From<&'a Arg> for &'a str {
        fn from(a: &'a Arg) -> Self {
            &a.0
        }
    }

    impl fmt::Display for Arg {
        fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
            write!(f, "{}", self.0)
        }
    }

    // #[derive(Debug, Clone, PartialEq, Eq, Default)]
    // enum Name {
    //     Echo,
    //     Self_,
    //     #[default]
    //     None,
    // }

    // impl TryFrom<&str> for Name {
    //     type Error = ();

    //     fn try_from(value: &str) -> Result<Self, Self::Error> {
    //         match value {
    //             "echo" => Ok(Self::Echo),
    //             PROGRAM_NAME => Ok(Self::Self_),
    //             _ => Err(()),
    //         }
    //     }
    // }

    // impl fmt::Display for Name {
    //     fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
    //         match self {
    //             Self::Echo => write!(f, "echo"),
    //             Self::Self_ => write!(f, "{}", PROGRAM_NAME),
    //             Self::None => write!(f, ""),
    //         }
    //     }
    // }

    // impl TryFrom<String> for ProgramName {
    //     type Error = ();

    //     fn try_from(value: String) -> Result<Self, Self::Error> {
    //         Self::try_from(value.as_str())
    //     }
    // }

    #[derive(Debug, Clone, PartialEq, Eq, Default)]
    struct Invocation {
        // name: Name,
        //
        flags: Vec<Flag>,
        // options: Vec<ProgramFlag>,
        args: Vec<Arg>,
        //
        stdin: Option<String>,
        stdout: Option<String>,
    }

    #[derive(Debug, Clone, PartialEq, Eq)]
    enum Program {
        Echo(Invocation),
        Self_(Invocation),
    }

    // enum ProgramCreationError {
    //     UnsupportedName,
    //     InvalidEscapeSequence,
    // }

    impl Program {
        fn new(name: &str, mut invocation: Invocation) -> Self {
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
                Program::Echo(_) => panic!("Echo cannot be run, only used to generate stdin"),
                Program::Self_(inv) => {
                    let mut cmd = Command::cargo_bin(name).unwrap();

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
                Program::Echo(_) => panic!("Echo should not be in the middle of a pipe"),
                Program::Self_(inv) => {
                    inv.stdin = Some(stdin.into());
                }
            }

            match chain
                .back_mut()
                .expect("No second program to assemble with")
            {
                Program::Echo(_) => panic!("Echo should not be at the end of a pipe"),
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

    // #[derive(Debug, Clone, PartialEq, Eq)]
    // struct EchoFlags {
    //     no_newline: bool,
    //     escape: bool,
    // }

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
    fn parse_command_output_pair(input: &str) -> IResult<&str, PipedPrograms> {
        let (input, _terminal_prompt) = char('$')(input)?;

        // let (input, echo) = invocation(input)?;
        let (input, _) = space0(input)?;
        // eprintln!("Parsed echo: {:#?}", echo);

        // let (input, _echo_cmd) = maybe_ws(tag("echo"))(input)?;

        // let mut echo_flags = None;
        // let (input, maybe_echo_option) = opt(char('-'))(input)?;
        // let input = match maybe_echo_option {
        //     Some(_) => {
        //         let (input, options) = ascii_alpha1(input)?;

        //         echo_flags = Some(EchoFlags {
        //             no_newline: options.contains('n'),
        //             escape: options.contains('e'),
        //         });

        //         let (input, _) = space0(input)?;

        //         input
        //     }
        //     None => input,
        // };

        // let (input, stdin) = parse_quoted(input)?;

        // let (input, _) = space0(input)?;

        let (input, cmds) = parse_pipe_components(input)?;
        eprintln!("Parsed commands: {:#?}", cmds);

        let (input, _) = take_until("\n")(input)?;
        let (input, _) = line_ending(input)?;

        let (input, stdout) = take_till(|c| c == '\0' || c == '$')(input)?;
        eprintln!("Parsed stdout: {:#?}", stdout);
        // let (input, _) = char('@')(input)?;

        let stdout = stdout.trim_end();
        // let (input, _) = space0(input)?;

        // let mut stdin = match echo_flags {
        //     Some(flags) => {
        //         let mut out = stdin.to_string();

        //         if flags.escape {
        //             out = unescape(stdin).ok_or(nom::Err::Error(nom::error::Error::new(
        //                 // Not really a great conversion but maybe prettier than panic?
        //                 input,
        //                 nom::error::ErrorKind::Escaped,
        //             )))?;
        //         }

        //         if flags.no_newline {
        //             out = out.trim_end().to_string();
        //         }

        //         Some(out)
        //     }

        //     None => Some(stdin.to_string()),
        // };

        // assert!(stdin.is_some(), "Should have a stdin by now");

        // let mut cuts = VecDeque::new();
        // for (_program_name, short_flags, long_flags, first_arg, second_arg) in cmds {
        //     let cut = ProgramInvocation {
        //         stdin,
        //         short_flags: short_flags.into_iter().map(String::from).collect(),
        //         long_flags: long_flags.into_iter().map(String::from).collect(),
        //         args: [first_arg, second_arg]
        //             .into_iter()
        //             .filter_map(|s| s.map(String::from))
        //             .collect(),
        //         stdout: None,
        //     };

        //     // Only first command has a stdin, rest won't have any.
        //     stdin = None;

        //     cuts.push(cut);
        // }

        // Analogously to how the initial command is the only one with stdin.
        // cuts.last_mut().unwrap().stdout = Some(stdout.to_string());

        Ok((
            input,
            PipedPrograms::assemble(cmds.into_iter(), stdout).expect("Should be able to assemble"),
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

    // fn invocation<'a, F: 'a, O, E: ParseError<&'a str>>(
    //     // inner: F,
    // ) -> impl FnMut(&str) -> IResult<(&'a str, Vec<&'a str>, Vec<&'a str>, Option<&'a str>, Option<&'a str>), O, E>
    // // where
    // //     F: Fn(&'a str) -> IResult<&'a str, O, E>,
    // {
    //     tuple((
    //         ws(ascii_alpha1), // Program name
    //         // Short flags precede long flags; this is hard-coded.
    //         many0(
    //             // Short flags, like `-s`, but also `-sGu`. No values.
    //             delimited(char('-'), ascii_alphanumeric1, space0),
    //         ),
    //         many0(delimited(
    //             // Long flags, like `--long-flag`. No values. Can contain hyphens
    //             // itself.
    //             tag("--"),
    //             take_while1(|c: char| c == '-' || c.is_ascii_alphanumeric()),
    //             space0,
    //         )),
    //         // First argument.
    //         opt(ws(parse_quoted)),
    //         // Second argument
    //         opt(ws(parse_quoted)),
    //     ))
    // }
    fn program(input: &str) -> IResult<&str, Program>
// where
        // nom::Err<E>: From<nom::Err<nom::error::Error<&'a str>>>,
        //     F: Fn(&'a str) -> IResult<&'a str, O, E>,
    {
        let inv = Rc::new(RefCell::new(Invocation::default()));

        let (input, (name, _)) = tuple((
            // map(
            maybe_ws(ascii_alpha1),
            //      |s| {
            //     inv.borrow_mut().name = s
            //         .try_into()
            //         .unwrap_or_else(|_| panic!("Unsupported program name: {s}"));

            //     s
            // }),
            many0(alt((
                map(
                    delimited(
                        // Long flags, like `--long-flag`. No values. Can contain hyphens
                        // itself.
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
                map(
                    // Quoted, positional arguments
                    maybe_ws(parse_quoted),
                    |s: &str| {
                        inv.borrow_mut().args.push(s.into());
                        s
                    },
                ),
            ))),
        ))(input)?;

        let (input, _) = space0(input)?;

        let program = Program::new(name, inv.borrow().clone());
        Ok((input, program))
    }

    // type Arg<'a> = Option<&'a str>;
    // type Options<'a> = Vec<&'a str>;
    fn parse_pipe_components(input: &str) -> IResult<&str, Vec<Program>> {
        // Our list isn't `A | B | C` but rather `| B | C` (`A` is `echo` and already
        // done). So kick off alternating list by eating initial pipe.
        // let (input, _) = char('|')(input)?;

        separated_list1(tag("|"), program)(input)
    }

    /// Parses multiple pairs of 'command and output' into a list of them.
    fn parse_code_blocks(input: &str) -> IResult<&str, Vec<PipedPrograms>> {
        many1(parse_command_output_pair)(input)
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

        let mut cut_pipes = Vec::new();
        let console = String::from("console");
        root.descendants().for_each(|node| {
            let value = node.to_owned().data.borrow().value.clone();

            if let NodeValue::CodeBlock(NodeCodeBlock { info, literal, .. }) = value {
                if info == console {
                    let (_, commands) = parse_code_blocks(&literal)
                        .finish()
                        .expect("Anything in `console` should be parseable as a command");
                    // println!("Found command to run: {:#?}", commands);
                    cut_pipes.extend(commands);
                }
            }
        });
        cut_pipes
    }

    #[test]
    fn test_readme_code_blocks() {
        let pipes = get_all_commands_under_test_pipes_from_readme();

        for pipe in pipes {
            let mut previous_stdin = None;
            for program in pipe {
                let mut cmd = Command::try_from(program.clone())
                    .expect("Should be able to convert invocation to cmd to run");

                // for flag in cut.short_flags {
                //     cmd.arg(format!("-{flag}"));
                // }

                // for flag in cut.long_flags {
                //     cmd.arg(format!("--{flag}"));
                // }

                // for arg in cut.args {
                //     cmd.arg(arg);
                // }

                // cmd.args(cut.short_flags.clone());

                // match (invocation.stdin, previous_stdin) {
                //     (Some(_), Some(_)) => {
                //         unreachable!("Cannot have initial and previous stdin simultaneously")
                //     }
                //     (Some(s), None) => {
                //         cmd.write_stdin(s);
                //     }
                //     (None, Some(p)) => {
                //         cmd.write_stdin(p);
                //     }
                //     (None, None) => unreachable!("Should have a stdin at all points"),
                // }

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
            // break;
        }
    }
}
