#[cfg(test)]
mod tests {
    use assert_cmd::Command;
    use comrak::{
        nodes::{NodeCodeBlock, NodeValue},
        parse_document, Arena, ComrakOptions,
    };

    use itertools::Itertools;
    use nom::{
        bytes::complete::{tag, take_till, take_until, take_while1},
        character::complete::{char, line_ending, multispace0},
        multi::many1,
        Finish, IResult,
    };

    #[derive(Debug, Clone, PartialEq, Eq)]
    struct CommandUnderTest {
        stdin: String,
        args: Vec<String>,
        stdout: String,
    }

    fn parse_command_output_pair(input: &str) -> IResult<&str, CommandUnderTest> {
        let (input, _) = char('$')(input)?;

        let (input, _) = multispace0(input)?;
        let (input, _) = tag("echo")(input)?;
        let (input, _) = multispace0(input)?;

        let quote = '\'';
        let (input, _) = char(quote)(input)?;
        // Doesn't handle escaping
        let (input, stdin) = take_until("\'")(input)?;
        let (input, _) = char(quote)(input)?;

        let (input, _) = multispace0(input)?;
        let (input, _) = char('|')(input)?;
        let (input, _) = multispace0(input)?;

        let (input, _program) = take_till(|c| c == ' ')(input)?;

        let (input, args) = take_while1(|c| c != '#' && c != '\n')(input)?;
        let (input, _) = take_until("\n")(input)?;
        let (input, _) = line_ending(input)?;

        let (input, stdout) = take_till(|c| c == '\0' || c == '$')(input)?;
        let stdout = stdout.trim_end();
        let (input, _) = multispace0(input)?;

        Ok((
            input,
            CommandUnderTest {
                stdin: stdin.trim().to_string(),
                args: args
                    .split_whitespace()
                    .map(String::from)
                    .filter(|s| !s.is_empty())
                    .map(|s| s.replace(quote, ""))
                    .collect_vec(),
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
                    let x = parse_code_blocks(&literal).finish();
                    println!("{:#?}", x);
                    let parsed = parse_code_blocks(&literal)
                        .finish()
                        .expect("Anything in `console` should be parseable as a command");
                    println!("{:#?}", parsed);
                    cuts.extend(parsed.1);
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
