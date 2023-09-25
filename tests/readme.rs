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
        multi::many1,
        sequence::delimited,
        Finish, IResult,
    };
    use shell_words::split;
    use unescape::unescape;

    #[derive(Debug, Clone, PartialEq, Eq)]
    struct CommandUnderTest {
        stdin: String,
        args: Vec<String>,
        stdout: String,
    }

    #[derive(Debug, Clone, PartialEq, Eq)]
    struct EchoFlags {
        no_newline: bool,
        escape: bool,
    }

    /// https://stackoverflow.com/a/58907488/11477374
    fn parse_quoted(input: &str) -> IResult<&str, &str> {
        let esc = escaped(none_of(r"'"), '\\', tag("'"));
        let esc_or_empty = alt((esc, tag("")));
        let res = delimited(tag("'"), esc_or_empty, tag("'"))(input)?;

        Ok(res)
    }

    fn parse_command_output_pair(input: &str) -> IResult<&str, CommandUnderTest> {
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

        let (input, _unix_pipe) = delimited(multispace0, char('|'), multispace0)(input)?;

        let (input, _program_name) = ascii_alpha1(input)?;

        let (input, _) = multispace0(input)?;

        let (input, raw_args) = take_while1(|c| c != '#' && c != '\n')(input)?;
        let (input, _) = take_until("\n")(input)?;
        let (input, _) = line_ending(input)?;

        let (input, stdout) = take_till(|c| c == '\0' || c == '$')(input)?;
        let stdout = stdout.trim_end();
        let (input, _) = multispace0(input)?;

        let stdin = match echo_flags {
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

                out
            }

            None => stdin.to_string(),
        };

        Ok((
            input,
            CommandUnderTest {
                stdin,
                args: split(raw_args).expect("Should be able to split args"),
                stdout: stdout.trim().to_string(),
            },
        ))
    }

    fn parse_code_blocks(input: &str) -> IResult<&str, Vec<CommandUnderTest>> {
        many1(parse_command_output_pair)(input)
    }

    fn get_all_commands_under_test_from_readme() -> Vec<CommandUnderTest> {
        let arena = Arena::new();

        let root = parse_document(
            &arena,
            include_str!("../README.md"),
            &ComrakOptions::default(),
        );

        let mut cuts = Vec::new();
        let console = String::from("console");
        root.descendants().for_each(|node| {
            let value = node.to_owned().data.borrow().value.clone();

            if let NodeValue::CodeBlock(NodeCodeBlock { info, literal, .. }) = value {
                if info == console {
                    let (_, commands) = parse_code_blocks(&literal)
                        .finish()
                        .expect("Anything in `console` should be parseable as a command");
                    println!("Found command to run: {:#?}", commands);
                    cuts.extend(commands);
                }
            }
        });
        cuts
    }

    #[test]
    fn test_readme_code_blocks() {
        let cuts = get_all_commands_under_test_from_readme();

        for cut in cuts {
            let mut cmd = Command::cargo_bin(env!("CARGO_PKG_NAME")).unwrap();
            cmd.args(cut.args.clone());
            cmd.write_stdin(cut.stdin);

            cmd.assert().success().stdout(cut.stdout);
        }
    }
}
