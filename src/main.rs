use betterletters::scoping::{
    langs::{
        python::{Python, PythonQuery},
        typescript::{TypeScript, TypeScriptQuery},
    },
    literal::Literal,
    ScopedViewBuildStep, ScoperBuildError,
};
#[cfg(feature = "deletion")]
use betterletters::stages::DeletionStage;
#[cfg(feature = "german")]
use betterletters::stages::GermanStage;
#[cfg(feature = "lower")]
use betterletters::stages::LowerStage;
#[cfg(feature = "normalization")]
use betterletters::stages::NormalizationStage;
#[cfg(feature = "replace")]
use betterletters::stages::ReplacementStage;
#[cfg(feature = "squeeze")]
use betterletters::stages::SqueezeStage;
#[cfg(feature = "titlecase")]
use betterletters::stages::TitlecaseStage;
#[cfg(feature = "upper")]
use betterletters::stages::UpperStage;
#[cfg(feature = "symbols")]
use betterletters::stages::{SymbolsInversionStage, SymbolsStage};
use betterletters::{apply, scoping::regex::Regex, Stage};
// use cli::PythonQuery;
use log::{debug, info, warn, LevelFilter};
use std::io::{self, Error, Read, Write};

fn main() -> Result<(), Error> {
    let args = cli::Cli::init();

    let level_filter = level_filter_from_env_and_verbosity(args.options.additional_verbosity);
    env_logger::Builder::new()
        .filter_level(level_filter)
        .format_timestamp_micros() // High precision is nice for benchmarks
        .init();

    info!("Launching app with args: {:?}", args);

    let scopers = match assemble_scopers(&args) {
        Ok(s) => s,
        Err(e) => match e {
            // Kinda abusive of these `io::ErrorKind`s...
            ScoperBuildError::RegexError(r) => {
                return Err(Error::new(io::ErrorKind::InvalidInput, r))
            }
            ScoperBuildError::LiteralError(l) => {
                return Err(Error::new(io::ErrorKind::InvalidInput, l))
            }
            ScoperBuildError::EmptyScope => {
                return Err(Error::new(
                    io::ErrorKind::InvalidInput,
                    "Empty scope is not allowed",
                ))
            }
        },
    };

    let stages = assemble_stages(&args).map_err(|e| Error::new(io::ErrorKind::InvalidInput, e))?;

    let mut buf = String::new();
    std::io::stdin().read_to_string(&mut buf)?;

    let result = apply(&buf, &scopers, &stages)?;

    let mut destination = io::stdout();
    destination.write_all(result.as_bytes())?;

    info!("Done, exiting");
    Ok(())
}

fn assemble_scopers(
    args: &cli::Cli,
) -> Result<Vec<Box<dyn ScopedViewBuildStep>>, ScoperBuildError> {
    let mut scopers: Vec<Box<dyn ScopedViewBuildStep>> = Vec::new();

    if let Some(python) = args.languages_scopes.python.clone() {
        if let Some(premade) = python.python {
            let query = PythonQuery::Premade(premade);

            scopers.push(Box::new(Python::new(query)));
        } else if let Some(custom) = python.python_query {
            let query = PythonQuery::Custom(custom);

            scopers.push(Box::new(Python::new(query)));
        }
    }

    if let Some(typescript) = args.languages_scopes.typescript.clone() {
        if let Some(premade) = typescript.typescript {
            let query = TypeScriptQuery::Premade(premade);

            scopers.push(Box::new(TypeScript::new(query)));
        } else if let Some(custom) = typescript.typescript_query {
            let query = TypeScriptQuery::Custom(custom);

            scopers.push(Box::new(TypeScript::new(query)));
        }
    }

    if args.options.literal_string {
        scopers.push(Box::new(Literal::try_from(args.scope.clone())?));
    } else {
        scopers.push(Box::new(Regex::try_from(args.scope.clone())?));
    }

    Ok(scopers)
}

fn assemble_stages(args: &cli::Cli) -> Result<Vec<Box<dyn Stage>>, String> {
    let mut stages: Vec<Box<dyn Stage>> = Vec::new();

    #[cfg(feature = "replace")]
    if let Some(replacement) = args.composable_stages.replace.clone() {
        stages.push(Box::new(ReplacementStage::try_from(replacement)?));
        debug!("Loaded stage: Replacement");
    }

    #[cfg(feature = "squeeze")]
    if args.standalone_stages.squeeze {
        stages.push(Box::<SqueezeStage>::default());
        debug!("Loaded stage: Squeeze");
    }

    #[cfg(feature = "german")]
    if args.composable_stages.german {
        stages.push(Box::new(GermanStage::new(
            // Smell? Bug if bools swapped.
            args.german_options.german_prefer_original,
            args.german_options.german_naive,
        )));
        debug!("Loaded stage: German");
    }

    #[cfg(feature = "symbols")]
    if args.composable_stages.symbols {
        if args.options.invert {
            stages.push(Box::<SymbolsInversionStage>::default());
            debug!("Loaded stage: SymbolsInversion");
        } else {
            stages.push(Box::<SymbolsStage>::default());
            debug!("Loaded stage: Symbols");
        }
    }

    #[cfg(feature = "deletion")]
    if args.standalone_stages.delete {
        stages.push(Box::<DeletionStage>::default());
        debug!("Loaded stage: Deletion");
    }

    #[cfg(feature = "upper")]
    if args.composable_stages.upper {
        stages.push(Box::<UpperStage>::default());
        debug!("Loaded stage: Upper");
    }

    #[cfg(feature = "lower")]
    if args.composable_stages.lower {
        stages.push(Box::<LowerStage>::default());
        debug!("Loaded stage: Lower");
    }

    #[cfg(feature = "titlecase")]
    if args.composable_stages.titlecase {
        stages.push(Box::<TitlecaseStage>::default());
        debug!("Loaded stage: Titlecase");
    }

    #[cfg(feature = "normalization")]
    if args.composable_stages.normalize {
        stages.push(Box::<NormalizationStage>::default());
        debug!("Loaded stage: Normalization");
    }

    if stages.is_empty() {
        // Doesn't hurt, but warn loudly
        warn!("No stages loaded, will return input unchanged");
    }

    Ok(stages)
}

/// To the default log level found in the environment, adds the requested additional
/// verbosity level, clamped to the maximum available.
///
/// See also
/// https://docs.rs/env_logger/latest/env_logger/struct.Env.html#default-environment-variables
/// and https://docs.rs/env_logger/latest/env_logger/#enabling-logging
fn level_filter_from_env_and_verbosity(additional_verbosity: u8) -> LevelFilter {
    let available = LevelFilter::iter().collect::<Vec<_>>();
    let default = env_logger::Builder::from_default_env().build().filter();

    let mut level = default as usize; // Implementation detail of `log` crate
    level += additional_verbosity as usize;

    available.get(level).copied().unwrap_or_else(|| {
        eprintln!("Requested additional verbosity on top of env default exceeds maximum, will use maximum");

        available
            .last()
            .copied()
            .expect("At least one level must be available")
    })
}

mod cli {
    use betterletters::{
        scoping::langs::{
            python::{CustomPythonQuery, PremadePythonQuery},
            typescript::{CustomTypeScriptQuery, PremadeTypeScriptQuery},
        },
        GLOBAL_SCOPE,
    };
    use clap::{builder::ArgPredicate, ArgAction, Parser};

    /// Main CLI entrypoint.
    ///
    /// Using `verbatim_doc_comment` a lot as otherwise lines wouldn't wrap neatly. I
    /// format them narrowly manually anyway, so can just use them verbatim.
    #[derive(Parser, Debug)]
    #[command(author, version, about, verbatim_doc_comment, long_about = None)]
    pub(super) struct Cli {
        /// Scope to apply to, as a regular expression pattern
        ///
        /// If string literal mode is requested, will be interpreted as a literal string.
        ///
        /// Stages will apply their transformations within this scope only.
        ///
        /// The default is the global scope, matching the entire input.
        ///
        /// Where that default is meaningless (e.g., deletion), this argument is
        /// _required_.
        #[arg(
            value_name = "SCOPE",
            default_value = GLOBAL_SCOPE,
            verbatim_doc_comment,
            default_value_if("literal_string", ArgPredicate::IsPresent, None)
        )]
        pub scope: String,

        #[command(flatten)]
        pub composable_stages: ComposableStages,

        #[command(flatten)]
        pub standalone_stages: StandaloneStages,

        #[command(flatten)]
        pub options: GlobalOptions,

        #[command(flatten)]
        pub languages_scopes: LanguageScopes,

        #[cfg(feature = "german")]
        #[command(flatten)]
        pub german_options: GermanStageOptions,
    }

    #[derive(Parser, Debug)]
    #[group(required = false, multiple = true)]
    #[command(next_help_heading = "Options (global)")]
    pub(super) struct GlobalOptions {
        /// Undo the effects of passed stages, where applicable
        ///
        /// Requires a 1:1 mapping (bijection) between replacements and original, which
        /// is currently available for:
        ///
        /// - symbols: '≠' <-> '!=' etc.
        ///
        /// Other stages:
        ///
        /// - german: inverting e.g. 'Ä' is ambiguous (can be 'Ae' or 'AE')
        ///
        /// - upper, lower, deletion, squeeze: inversion is impossible as information is
        ///   lost
        ///
        /// These may still be passed, but will be ignored for inversion and applied
        /// normally
        #[cfg(feature = "symbols")]
        #[arg(short, long, env, requires = "symbols", verbatim_doc_comment)]
        pub invert: bool,
        /// Do not interpret the scope as a regex. Instead, interpret it as a literal
        /// string. Will require a scope to be passed.
        #[arg(short('L'), long, env, verbatim_doc_comment)]
        pub literal_string: bool,
        /// Increase log verbosity level
        ///
        /// The base log level to use is read from the `RUST_LOG` environment variable
        /// (if missing, 'error'), and increased according to the number of times this
        /// flag is given.
        #[arg(
            short = 'v',
            long = "verbose",
            action = ArgAction::Count,
            verbatim_doc_comment
        )]
        pub additional_verbosity: u8,
    }

    #[derive(Parser, Debug)]
    #[group(required = false, multiple = true)]
    #[command(next_help_heading = "Composable Stages")]
    pub(super) struct ComposableStages {
        /// Replace scope by this (fixed) value
        ///
        /// Specially treated stage for ergonomics and compatibility with `tr`.
        ///
        /// If given, will run before any other stage.
        #[cfg(feature = "replace")]
        #[arg(value_name = "REPLACEMENT", env, verbatim_doc_comment)]
        pub replace: Option<String>,
        /// Uppercase scope
        #[cfg(feature = "upper")]
        #[arg(short, long, env, verbatim_doc_comment)]
        pub upper: bool,
        /// Lowercase scope
        #[cfg(feature = "lower")]
        #[arg(short, long, env, verbatim_doc_comment)]
        pub lower: bool,
        /// Titlecase scope
        #[cfg(feature = "titlecase")]
        #[arg(short, long, env, verbatim_doc_comment)]
        pub titlecase: bool,
        /// Normalize (Normalization Form D) scope, and throw away marks
        #[cfg(feature = "normalization")]
        #[arg(short, long, env, verbatim_doc_comment)]
        pub normalize: bool,
        /// Perform substitutions on German words, such as 'Abenteuergruesse' to
        /// 'Abenteuergrüße'
        ///
        /// ASCII spellings for Umlauts (ae, oe, ue) and Eszett (ss) are replaced by
        /// their respective native Unicode (ä, ö, ü, ß).
        ///
        /// Arbitrary compound words are supported.
        ///
        /// Words legally containing alternative spellings are not modified.
        ///
        /// Words require correct spelling to be detected.
        #[cfg(feature = "german")]
        #[arg(short, long, verbatim_doc_comment)]
        pub german: bool,
        /// Perform substitutions on symbols, such as '!=' to '≠', '->' to '→'
        ///
        /// Helps translate 'ASCII art' into native Unicode representations.
        #[cfg(feature = "symbols")]
        #[arg(short = 'S', long, verbatim_doc_comment)]
        pub symbols: bool,
    }

    #[derive(Parser, Debug)]
    #[group(required = false, multiple = false)]
    #[command(next_help_heading = "Standalone Stages (only usable alone)")]
    pub(super) struct StandaloneStages {
        /// Delete scope
        ///
        /// Cannot be used with any other stage: no point in deleting and performing any
        /// other action. Sibling stages would either receive empty input or have their
        /// work wiped.
        #[cfg(feature = "deletion")]
        #[arg(
            short,
            long,
            requires = "scope",
            conflicts_with = stringify!(ComposableStages),
            verbatim_doc_comment
        )]
        pub delete: bool,
        /// Squeeze consecutive occurrences of scope into one
        ///
        /// For example, 'a++b' -> 'a+b' for a scope of '+'.
        ///
        /// Quantifiers in scope will have their greediness inverted, allowing for
        /// 'A1337B' -> 'A1B' for a scope of '\d+' (no '?' required).
        ///
        /// A greedy scope ('\d+?') would match all of '1337' and replace nothing.
        #[cfg(feature = "squeeze")]
        #[arg(short, long, env, requires = "scope", verbatim_doc_comment)]
        pub squeeze: bool,
    }

    #[derive(Parser, Debug)]
    #[group(required = false, multiple = false)]
    #[command(next_help_heading = "Language scopes")]
    pub(super) struct LanguageScopes {
        #[command(flatten)]
        pub python: Option<PythonScope>,
        #[command(flatten)]
        pub typescript: Option<TypeScriptScope>,
    }

    #[derive(Parser, Debug, Clone)]
    #[group(required = false, multiple = false)]
    pub(super) struct PythonScope {
        /// Scope Python code using a premade query.
        #[arg(long, env, verbatim_doc_comment)]
        pub python: Option<PremadePythonQuery>,

        /// Scope Python code using a custom tree-sitter query.
        #[arg(long, env, verbatim_doc_comment)]
        pub python_query: Option<CustomPythonQuery>,
    }

    #[derive(Parser, Debug, Clone)]
    #[group(required = false, multiple = false)]
    pub(super) struct TypeScriptScope {
        /// Scope TypeScript code using a premade query.
        #[arg(long, env, verbatim_doc_comment)]
        pub typescript: Option<PremadeTypeScriptQuery>,

        /// Scope TypeScript code using a custom tree-sitter query.
        #[arg(long, env, verbatim_doc_comment)]
        pub typescript_query: Option<CustomTypeScriptQuery>,
    }

    #[cfg(feature = "german")]
    #[derive(Parser, Debug)]
    #[group(required = false, multiple = true)]
    #[command(next_help_heading = "Options (german)")]
    pub(super) struct GermanStageOptions {
        /// When some original version and its replacement are equally legal, prefer the
        /// original and do not modify.
        ///
        /// For example, "Busse" (original) and "Buße" (replacement) are equally legal
        /// words: by default, the tool would prefer the latter.
        // More fine-grained control is not available. We are not in the business of
        // natural language processing or LLMs, so that's all we can offer...
        #[arg(long, env, verbatim_doc_comment)]
        pub german_prefer_original: bool,
        /// Always perform any possible replacement ('ae' -> 'ä', 'ss' -> 'ß', etc.),
        /// regardless of legality of the resulting word
        ///
        /// Useful for names, which are otherwise not modifiable as they do not occur in
        /// dictionaries. Called 'naive' as this does not perform legal checks.
        #[arg(long, env, verbatim_doc_comment)]
        pub german_naive: bool,
    }

    impl Cli {
        pub(super) fn init() -> Self {
            Self::parse()
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use env_logger::DEFAULT_FILTER_ENV;
    use log::LevelFilter;
    use rstest::rstest;
    use serial_test::serial;
    use std::env;

    #[rstest]
    #[case(None, 0, LevelFilter::Error)]
    #[case(None, 1, LevelFilter::Warn)]
    #[case(None, 2, LevelFilter::Info)]
    #[case(None, 3, LevelFilter::Debug)]
    #[case(None, 4, LevelFilter::Trace)]
    #[case(None, 5, LevelFilter::Trace)]
    #[case(None, 128, LevelFilter::Trace)]
    //
    #[case(Some("off"), 0, LevelFilter::Off)]
    #[case(Some("off"), 1, LevelFilter::Error)]
    #[case(Some("off"), 2, LevelFilter::Warn)]
    #[case(Some("off"), 3, LevelFilter::Info)]
    #[case(Some("off"), 4, LevelFilter::Debug)]
    #[case(Some("off"), 5, LevelFilter::Trace)]
    #[case(Some("off"), 6, LevelFilter::Trace)]
    #[case(Some("off"), 128, LevelFilter::Trace)]
    //
    #[case(Some("error"), 0, LevelFilter::Error)]
    #[case(Some("error"), 1, LevelFilter::Warn)]
    #[case(Some("error"), 2, LevelFilter::Info)]
    #[case(Some("error"), 3, LevelFilter::Debug)]
    #[case(Some("error"), 4, LevelFilter::Trace)]
    #[case(Some("error"), 5, LevelFilter::Trace)]
    #[case(Some("error"), 128, LevelFilter::Trace)]
    //
    #[case(Some("warn"), 0, LevelFilter::Warn)]
    #[case(Some("warn"), 1, LevelFilter::Info)]
    #[case(Some("warn"), 2, LevelFilter::Debug)]
    #[case(Some("warn"), 3, LevelFilter::Trace)]
    #[case(Some("warn"), 4, LevelFilter::Trace)]
    #[case(Some("warn"), 128, LevelFilter::Trace)]
    //
    #[case(Some("info"), 0, LevelFilter::Info)]
    #[case(Some("info"), 1, LevelFilter::Debug)]
    #[case(Some("info"), 2, LevelFilter::Trace)]
    #[case(Some("info"), 3, LevelFilter::Trace)]
    #[case(Some("info"), 128, LevelFilter::Trace)]
    //
    #[case(Some("debug"), 0, LevelFilter::Debug)]
    #[case(Some("debug"), 1, LevelFilter::Trace)]
    #[case(Some("debug"), 2, LevelFilter::Trace)]
    #[case(Some("debug"), 128, LevelFilter::Trace)]
    //
    #[case(Some("trace"), 0, LevelFilter::Trace)]
    #[case(Some("trace"), 1, LevelFilter::Trace)]
    #[case(Some("trace"), 128, LevelFilter::Trace)]
    //
    #[serial] // This is multi-threaded, but env var access might not be thread-safe
    fn test_level_filter_from_env_and_verbosity(
        #[case] env_value: Option<&str>,
        #[case] additional_verbosity: u8,
        #[case] expected: LevelFilter,
    ) {
        if let Some(env_value) = env_value {
            env::set_var(DEFAULT_FILTER_ENV, env_value);
        } else {
            // Might be set on parent and fork()ed down
            env::remove_var(DEFAULT_FILTER_ENV);
        }

        // Sanity check for sequential tests
        let i_am_not_sure_if_this_test_really_runs_sequentially = false;
        if i_am_not_sure_if_this_test_really_runs_sequentially {
            std::thread::sleep(std::time::Duration::from_secs(2));
        }

        let result = level_filter_from_env_and_verbosity(additional_verbosity);
        assert_eq!(result, expected);
    }
}
