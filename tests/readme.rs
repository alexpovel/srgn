#[cfg(test)]
mod tests {
    use assert_cmd::Command;
    use comrak::{
        nodes::{NodeCodeBlock, NodeValue},
        parse_document, Arena, ComrakOptions,
    };

    use nom::{
        branch::alt,
        bytes::complete::{escaped, tag, take_till, take_until, take_while1},
        character::complete::{alpha1 as ascii_alpha1, char, line_ending, multispace0, none_of},
        combinator::opt,
        multi::{many1, separated_list1},
        sequence::{delimited, tuple},
        Finish, IResult,
    };
    use shell_words::split;
    use unescape::unescape;

    #[derive(Debug, Clone, PartialEq, Eq)]
    struct CommandUnderTest {
        stdin: Option<String>,
        args: Vec<String>,
        stdout: Option<String>,
    }

    /// Multiple commands can be piped together.
    #[derive(Debug, Clone, PartialEq, Eq)]
    struct CommandUnderTestPipe(Vec<CommandUnderTest>);

    #[derive(Debug, Clone, PartialEq, Eq)]
    struct EchoFlags {
        no_newline: bool,
        escape: bool,
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
    fn parse_command_output_pair(input: &str) -> IResult<&str, CommandUnderTestPipe> {
        let (input, _terminal_prompt) = char('$')(input)?;

        let (input, _echo_cmd) = delimited(multispace0, tag("echo"), multispace0)(input)?;

        let mut echo_flags = None;
        let (input, maybe_echo_option) = opt(char('-'))(input)?;
        let input = match maybe_echo_option {
            Some(_) => {
                let (input, options) = ascii_alpha1(input)?;

                echo_flags = Some(EchoFlags {
                    no_newline: options.contains('n'),
                    escape: options.contains('e'),
                });

                let (input, _) = multispace0(input)?;

                input
            }
            None => input,
        };

        let (input, stdin) = parse_quoted(input)?;

        let (input, _) = multispace0(input)?;

        let (input, cmds) = parse_pipe_components(input)?;

        let (input, _) = take_until("\n")(input)?;
        let (input, _) = line_ending(input)?;

        let (input, stdout) = take_till(|c| c == '\0' || c == '$')(input)?;
        let stdout = stdout.trim_end();
        let (input, _) = multispace0(input)?;

        let mut stdin = match echo_flags {
            Some(flags) => {
                let mut out = stdin.to_string();

                if flags.escape {
                    out = unescape(stdin).ok_or(nom::Err::Error(nom::error::Error::new(
                        // Not really a great conversion but maybe prettier than panic?
                        input,
                        nom::error::ErrorKind::Escaped,
                    )))?;
                }

                if flags.no_newline {
                    out = out.trim_end().to_string();
                }

                Some(out)
            }

            None => Some(stdin.to_string()),
        };

        assert!(stdin.is_some(), "Should have a stdin by now");

        let mut cuts = Vec::new();
        for (_, _program_name, _, raw_args) in cmds {
            eprintln!("raw_args: {}", raw_args);

            let cut = CommandUnderTest {
                stdin,
                args: split(raw_args).expect("Should be able to split args"),
                stdout: None,
            };

            // Only first command has a stdin, rest won't have any.
            stdin = None;

            cuts.push(cut);
        }

        // Analogously to how the initial command is the only one with stdin.
        cuts.last_mut().unwrap().stdout = Some(stdout.to_string());

        Ok((input, CommandUnderTestPipe(cuts)))
    }

    fn parse_pipe_components(input: &str) -> IResult<&str, Vec<(&str, &str, &str, &str)>> {
        // Our list isn't `A | B | C` but rather `| B | C` (`A` is `echo` and already
        // done). So kick off alternating list by eating initial pipe.
        let (input, _) = char('|')(input)?;

        separated_list1(
            tag("|"),
            tuple((
                multispace0,
                ascii_alpha1, // program name
                multispace0,
                take_while1(|c| c != '#' && c != '\n' && c != '|'), // args
            )),
        )(input)
    }

    /// Parses multiple pairs of 'command and output' into a list of them.
    fn parse_code_blocks(input: &str) -> IResult<&str, Vec<CommandUnderTestPipe>> {
        many1(parse_command_output_pair)(input)
    }

    /// https://stackoverflow.com/a/58907488/11477374
    fn parse_quoted(input: &str) -> IResult<&str, &str> {
        let esc = escaped(none_of(r"'"), '\\', tag("'"));
        let esc_or_empty = alt((esc, tag("")));
        let res = delimited(tag("'"), esc_or_empty, tag("'"))(input)?;

        Ok(res)
    }

    fn get_all_commands_under_test_pipes_from_readme() -> Vec<CommandUnderTestPipe> {
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
                    println!("Found command to run: {:#?}", commands);
                    cut_pipes.extend(commands);
                }
            }
        });
        cut_pipes
    }

    #[test]
    fn test_readme_code_blocks() {
        let cut_pipes = get_all_commands_under_test_pipes_from_readme();

        for cut_pipe in cut_pipes {
            let cuts = cut_pipe.0;

            let mut previous_stdin = None;
            for cut in cuts {
                let mut cmd = Command::cargo_bin(env!("CARGO_PKG_NAME")).unwrap();

                cmd.args(cut.args.clone());

                match (cut.stdin, previous_stdin) {
                    (Some(_), Some(_)) => {
                        unreachable!("Cannot have initial and previous stdin simultaneously")
                    }
                    (Some(s), None) => {
                        cmd.write_stdin(s);
                    }
                    (None, Some(p)) => {
                        cmd.write_stdin(p);
                    }
                    (None, None) => unreachable!("Should have a stdin at all points"),
                }

                if let Some(stdout) = cut.stdout {
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
