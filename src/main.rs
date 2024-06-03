use anyhow::{Context, Result};
use log::{debug, error, info, warn, LevelFilter};
use rayon::prelude::*;
use srgn::actions::Deletion;
#[cfg(feature = "german")]
use srgn::actions::German;
use srgn::actions::Lower;
use srgn::actions::Normalization;
use srgn::actions::Replacement;
use srgn::actions::Titlecase;
use srgn::actions::Upper;
#[cfg(feature = "symbols")]
use srgn::actions::{Symbols, SymbolsInversion};
use srgn::scoping::literal::LiteralError;
use srgn::scoping::regex::RegexError;
use srgn::{
    actions::Action,
    scoping::{
        langs::{
            csharp::{CSharp, CSharpQuery},
            go::{Go, GoQuery},
            python::{Python, PythonQuery},
            rust::{Rust, RustQuery},
            typescript::{TypeScript, TypeScriptQuery},
        },
        literal::Literal,
        regex::Regex,
        view::ScopedViewBuilder,
        Scoper,
    },
};
use std::{
    error::Error,
    fmt,
    fs::File,
    io::{self, IoSlice, Write},
};

fn main() -> Result<()> {
    let args = cli::Cli::init();

    let level_filter = level_filter_from_env_and_verbosity(args.options.additional_verbosity);
    env_logger::Builder::new()
        .filter_level(level_filter)
        .format_timestamp_micros() // High precision is nice for benchmarks
        .init();

    if let Some(shell) = args.shell {
        debug!("Generating completions file for {shell:?}.");
        cli::print_completions(shell, &mut cli::Cli::command());
        debug!("Done generating completions file, exiting.");

        return Ok(());
    }

    info!("Launching app with args: {:?}", args);

    debug!("Assembling scopers.");
    let scopers = assemble_scopers(&args)?;
    debug!("Done assembling scopers.");

    debug!("Assembling actions.");
    let actions = assemble_actions(&args)?;
    debug!("Done assembling actions.");

    match &args.options.files {
        Some(pattern) => {
            info!("Will use glob pattern: {:?}", pattern);

            let paths = glob::glob(pattern.as_str())
                .expect("Pattern is valid, as it's been compiled")
                .par_bridge()
                .map(|glob| {
                    let path = glob.context("Failed to glob")?;
                    debug!("Processing path: {:?}", path);

                    if !path.is_file() {
                        warn!(
                            "Path does not look like a file (will still try): '{}' (Metadata: {:?})",
                            path.display(),
                            path.metadata()
                        );
                    }

                    let contents = {
                        let file = File::open(&path)
                            .with_context(|| format!("Failed to read file: {:?}", path))?;

                        let mut source = std::io::BufReader::new(file);
                        let mut destination = std::io::Cursor::new(Vec::new());

                        apply(
                            &mut source,
                            &mut destination,
                            &scopers,
                            &actions,
                            args.options.fail_none,
                            args.options.fail_any,
                            args.standalone_actions.squeeze,
                        )
                        .with_context(|| format!("Failed to process file contents: {:?}", path))?;

                        destination.into_inner()
                    };

                    debug!("Got new file contents, writing to file: {:?}", path);
                    let mut file = File::create(&path)
                        .with_context(|| format!("Failed to truncate file: {:?}", path))?;
                    file.write_all(&contents)
                        .with_context(|| format!("Failed to write to file: {:?}", path))?;
                    debug!("Done processing file: {:?}", path);

                    {
                        let path_repr = path.display().to_string();
                        let slices = &[path_repr.as_bytes(), b"\n"].map(IoSlice::new);

                        std::io::stdout()
                            .lock()
                            .write_vectored(slices)
                            .context("Failed writing processed file's name to stdout")?;
                    }

                    Ok(path)
                })
                .collect::<Result<Vec<_>>>()
                .context("Failure in processing of files")?;

            if args.options.fail_empty_glob && paths.is_empty() {
                return Err(ApplicationError::EmptyGlob(pattern.clone()))
                    .context("No files processed");
            }
        }
        None => {
            info!("Will use stdin to stdout");
            let mut source = std::io::stdin().lock();
            let mut destination = std::io::stdout().lock();

            apply(
                &mut source,
                &mut destination,
                &scopers,
                &actions,
                args.options.fail_none,
                args.options.fail_any,
                args.standalone_actions.squeeze,
            )
            .context("Failed to process stdin")?;
        }
    }

    info!("Done, exiting");
    Ok(())
}

fn apply(
    source: &mut impl io::BufRead,
    destination: &mut impl io::Write,
    scopers: &Vec<Box<dyn Scoper>>,
    actions: &Vec<Box<dyn Action>>,
    fail_none: bool,
    fail_any: bool,
    squeeze: bool,
) -> Result<()> {
    // Streaming (e.g., line-based) wouldn't be too bad, and much more memory-efficient,
    // but language grammar-aware scoping needs entire files for context. Single lines
    // wouldn't do. There's no smart way of streaming that I can think of (where would
    // one break?).
    debug!("Reading entire source to string.");
    let mut buf = String::new();
    source
        .read_to_string(&mut buf)
        .context("Failed reading in source")?;
    debug!("Done reading source.");

    debug!("Building view.");
    let mut builder = ScopedViewBuilder::new(&buf);
    for scoper in scopers {
        builder.explode(scoper);
    }
    let mut view = builder.build();
    debug!("Done building view: {view:?}");

    if fail_none && !view.has_any_in_scope() {
        return Err(ApplicationError::NoneInScope.into());
    }

    if fail_any && view.has_any_in_scope() {
        return Err(ApplicationError::SomeInScope.into());
    };

    debug!("Applying actions to view.");
    let result = {
        if squeeze {
            view.squeeze();
        }

        for action in actions {
            view.map_with_context(action)?;
        }

        view.to_string()
    };
    debug!("Done applying actions to view.");

    debug!("Writing to destination.");
    destination
        .write_all(result.as_bytes())
        .context("Failed writing to destination")?;
    debug!("Done writing to destination.");

    Ok(())
}

#[derive(Debug)]
enum ApplicationError {
    SomeInScope,
    NoneInScope,
    EmptyGlob(glob::Pattern),
}

impl fmt::Display for ApplicationError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::SomeInScope => write!(
                f,
                "Some input was in scope, and explicit failure requested."
            ),
            Self::NoneInScope => write!(f, "Nothing in scope and explicit failure requested."),
            Self::EmptyGlob(p) => write!(f, "No files matched glob pattern: {:?}", p),
        }
    }
}

impl Error for ApplicationError {}

#[derive(Debug)]
pub enum ScoperBuildError {
    EmptyScope,
    RegexError(RegexError),
    LiteralError(LiteralError),
}

impl From<LiteralError> for ScoperBuildError {
    fn from(e: LiteralError) -> Self {
        Self::LiteralError(e)
    }
}

impl From<RegexError> for ScoperBuildError {
    fn from(e: RegexError) -> Self {
        Self::RegexError(e)
    }
}

impl fmt::Display for ScoperBuildError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::EmptyScope => write!(f, "Empty scope"),
            Self::RegexError(e) => write!(f, "Regex error: {}", e),
            Self::LiteralError(e) => write!(f, "Literal error: {}", e),
        }
    }
}

impl Error for ScoperBuildError {}

fn assemble_scopers(args: &cli::Cli) -> Result<Vec<Box<dyn Scoper>>> {
    let mut scopers: Vec<Box<dyn Scoper>> = Vec::new();

    if let Some(csharp) = args.languages_scopes.csharp.clone() {
        if let Some(premade) = csharp.csharp {
            let query = CSharpQuery::Premade(premade);

            scopers.push(Box::new(CSharp::new(query)));
        } else if let Some(custom) = csharp.csharp_query {
            let query = CSharpQuery::Custom(custom);

            scopers.push(Box::new(CSharp::new(query)));
        }
    }

    if let Some(go) = args.languages_scopes.go.clone() {
        if let Some(premade) = go.go {
            let query = GoQuery::Premade(premade);

            scopers.push(Box::new(Go::new(query)));
        } else if let Some(custom) = go.go_query {
            let query = GoQuery::Custom(custom);

            scopers.push(Box::new(Go::new(query)));
        }
    }

    if let Some(python) = args.languages_scopes.python.clone() {
        if let Some(premade) = python.python {
            let query = PythonQuery::Premade(premade);

            scopers.push(Box::new(Python::new(query)));
        } else if let Some(custom) = python.python_query {
            let query = PythonQuery::Custom(custom);

            scopers.push(Box::new(Python::new(query)));
        }
    }

    if let Some(rust) = args.languages_scopes.rust.clone() {
        if let Some(premade) = rust.rust {
            let query = RustQuery::Premade(premade);

            scopers.push(Box::new(Rust::new(query)));
        } else if let Some(custom) = rust.rust_query {
            let query = RustQuery::Custom(custom);

            scopers.push(Box::new(Rust::new(query)));
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
        scopers.push(Box::new(
            Literal::try_from(args.scope.clone()).context("Failed building literal string")?,
        ));
    } else {
        scopers.push(Box::new(
            Regex::try_from(args.scope.clone()).context("Failed building regex")?,
        ));
    }

    Ok(scopers)
}

fn assemble_actions(args: &cli::Cli) -> Result<Vec<Box<dyn Action>>> {
    let mut actions: Vec<Box<dyn Action>> = Vec::new();

    if let Some(replacement) = args.composable_actions.replace.clone() {
        actions.push(Box::new(
            Replacement::try_from(replacement).context("Failed building replacement string")?,
        ));
        debug!("Loaded action: Replacement");
    }

    #[cfg(feature = "german")]
    if args.composable_actions.german {
        actions.push(Box::new(German::new(
            // Smell? Bug if bools swapped.
            args.german_options.german_prefer_original,
            args.german_options.german_naive,
        )));
        debug!("Loaded action: German");
    }

    #[cfg(feature = "symbols")]
    if args.composable_actions.symbols {
        if args.options.invert {
            actions.push(Box::<SymbolsInversion>::default());
            debug!("Loaded action: SymbolsInversion");
        } else {
            actions.push(Box::<Symbols>::default());
            debug!("Loaded action: Symbols");
        }
    }

    if args.standalone_actions.delete {
        actions.push(Box::<Deletion>::default());
        debug!("Loaded action: Deletion");
    }

    if args.composable_actions.upper {
        actions.push(Box::<Upper>::default());
        debug!("Loaded action: Upper");
    }

    if args.composable_actions.lower {
        actions.push(Box::<Lower>::default());
        debug!("Loaded action: Lower");
    }

    if args.composable_actions.titlecase {
        actions.push(Box::<Titlecase>::default());
        debug!("Loaded action: Titlecase");
    }

    if args.composable_actions.normalize {
        actions.push(Box::<Normalization>::default());
        debug!("Loaded action: Normalization");
    }

    if actions.is_empty() && !(args.options.fail_any || args.options.fail_none) {
        // Doesn't hurt, but warn loudly
        error!("No actions loaded, will return input unchanged");
    }

    Ok(actions)
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
    use clap::{builder::ArgPredicate, ArgAction, Command, CommandFactory, Parser};
    use clap_complete::{generate, Generator, Shell};
    use srgn::{
        scoping::langs::{
            csharp::{CustomCSharpQuery, PremadeCSharpQuery},
            go::{CustomGoQuery, PremadeGoQuery},
            python::{CustomPythonQuery, PremadePythonQuery},
            rust::{CustomRustQuery, PremadeRustQuery},
            typescript::{CustomTypeScriptQuery, PremadeTypeScriptQuery},
        },
        GLOBAL_SCOPE,
    };

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
        /// Actions will apply their transformations within this scope only.
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

        /// Print shell completions for the given shell
        // This thing needs to live up here to show up within `Options` next to `--help`
        // and `--version`. Further down, it'd show up in the wrong section because we
        // alter `next_help_heading`.
        #[arg(long = "completions", value_enum, verbatim_doc_comment)]
        pub shell: Option<Shell>,

        #[command(flatten)]
        pub composable_actions: ComposableActions,

        #[command(flatten)]
        pub standalone_actions: StandaloneActions,

        #[command(flatten)]
        pub options: GlobalOptions,

        #[command(flatten)]
        pub languages_scopes: LanguageScopes,

        #[cfg(feature = "german")]
        #[command(flatten)]
        pub german_options: GermanOptions,
    }

    /// https://github.com/clap-rs/clap/blob/f65d421607ba16c3175ffe76a20820f123b6c4cb/clap_complete/examples/completion-derive.rs#L69
    pub(super) fn print_completions<G: Generator>(gen: G, cmd: &mut Command) {
        generate(gen, cmd, cmd.get_name().to_string(), &mut std::io::stdout());
    }

    #[derive(Parser, Debug)]
    #[group(required = false, multiple = true)]
    #[command(next_help_heading = "Options (global)")]
    pub(super) struct GlobalOptions {
        /// Glob of files to work on (instead of reading stdin).
        ///
        /// If processing occurs, it is done in-place, overwriting originals.
        ///
        /// For supported glob syntax, see:
        /// https://docs.rs/glob/0.3.1/glob/struct.Pattern.html
        ///
        /// Names of processed files are written to stdout.
        #[arg(long, verbatim_doc_comment)]
        pub files: Option<glob::Pattern>,
        /// Fail if file globbing is requested but returns no matches.
        #[arg(long, verbatim_doc_comment, requires = "files")]
        pub fail_empty_glob: bool,
        /// Undo the effects of passed actions, where applicable
        ///
        /// Requires a 1:1 mapping (bijection) between replacements and original, which
        /// is currently available for:
        ///
        /// - symbols: '≠' <-> '!=' etc.
        ///
        /// Other actions:
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
        /// If anything at all is found to be in scope, fail.
        ///
        /// The default is to continue processing normally.
        #[arg(long, verbatim_doc_comment)]
        pub fail_any: bool,
        /// If nothing is found to be in scope, fail.
        ///
        /// The default is to return the input unchanged (without failure).
        #[arg(long, verbatim_doc_comment)]
        pub fail_none: bool,
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
    #[command(next_help_heading = "Composable Actions")]
    pub(super) struct ComposableActions {
        /// Replace scope by this (fixed) value
        ///
        /// Specially treated action for ergonomics and compatibility with `tr`.
        ///
        /// If given, will run before any other action.
        #[arg(value_name = "REPLACEMENT", env, verbatim_doc_comment)]
        pub replace: Option<String>,
        /// Uppercase scope
        #[arg(short, long, env, verbatim_doc_comment)]
        pub upper: bool,
        /// Lowercase scope
        #[arg(short, long, env, verbatim_doc_comment)]
        pub lower: bool,
        /// Titlecase scope
        #[arg(short, long, env, verbatim_doc_comment)]
        pub titlecase: bool,
        /// Normalize (Normalization Form D) scope, and throw away marks
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
        #[arg(
            short,
            long,
            verbatim_doc_comment,
            // `true` as string is very ugly, but there's no other way?
            default_value_if("german-opts", ArgPredicate::IsPresent, "true")
        )]
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
    #[command(next_help_heading = "Standalone Actions (only usable alone)")]
    pub(super) struct StandaloneActions {
        /// Delete scope
        ///
        /// Cannot be used with any other action: no point in deleting and performing any
        /// other action. Sibling actions would either receive empty input or have their
        /// work wiped.
        #[arg(
            short,
            long,
            requires = "scope",
            conflicts_with = stringify!(ComposableActions),
            verbatim_doc_comment
        )]
        pub delete: bool,
        /// Squeeze consecutive occurrences of scope into one
        #[arg(
            short,
            long,
            visible_alias("squeeze-repeats"),
            env,
            requires = "scope",
            verbatim_doc_comment
        )]
        pub squeeze: bool,
    }

    #[derive(Parser, Debug)]
    #[group(required = false, multiple = false)]
    #[command(next_help_heading = "Language scopes")]
    pub(super) struct LanguageScopes {
        #[command(flatten)]
        pub csharp: Option<CSharpScope>,
        #[command(flatten)]
        pub go: Option<GoScope>,
        #[command(flatten)]
        pub python: Option<PythonScope>,
        #[command(flatten)]
        pub rust: Option<RustScope>,
        #[command(flatten)]
        pub typescript: Option<TypeScriptScope>,
    }

    #[derive(Parser, Debug, Clone)]
    #[group(required = false, multiple = false)]
    pub(super) struct CSharpScope {
        /// Scope CSharp code using a premade query.
        #[arg(long, env, verbatim_doc_comment)]
        pub csharp: Option<PremadeCSharpQuery>,

        /// Scope CSharp code using a custom tree-sitter query.
        #[arg(long, env, verbatim_doc_comment)]
        pub csharp_query: Option<CustomCSharpQuery>,
    }

    #[derive(Parser, Debug, Clone)]
    #[group(required = false, multiple = false)]
    pub(super) struct GoScope {
        /// Scope Go code using a premade query.
        #[arg(long, env, verbatim_doc_comment)]
        pub go: Option<PremadeGoQuery>,

        /// Scope Go code using a custom tree-sitter query.
        #[arg(long, env, verbatim_doc_comment)]
        pub go_query: Option<CustomGoQuery>,
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
    pub(super) struct RustScope {
        /// Scope Rust code using a premade query.
        #[arg(long, env, verbatim_doc_comment)]
        pub rust: Option<PremadeRustQuery>,

        /// Scope Rust code using a custom tree-sitter query.
        #[arg(long, env, verbatim_doc_comment)]
        pub rust_query: Option<CustomRustQuery>,
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
    #[group(required = false, multiple = true, id("german-opts"))]
    #[command(next_help_heading = "Options (german)")]
    pub(super) struct GermanOptions {
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

        pub(super) fn command() -> clap::Command {
            <Self as CommandFactory>::command()
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
