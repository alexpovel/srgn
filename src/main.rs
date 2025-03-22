//! The main entrypoint to `srgn` as a CLI application.
//!
//! It mainly draws from `srgn`, the library, for actual implementations. This file then
//! deals with CLI argument handling, I/O, threading, and more.

use std::error::Error;
use std::fs::{self, File};
use std::io::{self, Read, Write, stdout};
use std::path::{Path, PathBuf};
use std::sync::{Arc, Mutex};
use std::{env, fmt};

use anyhow::{Context, Result};
use colored::Colorize;
use ignore::{WalkBuilder, WalkState};
use itertools::Itertools;
use log::{LevelFilter, debug, error, info, trace, warn};
use pathdiff::diff_paths;
#[cfg(feature = "german")]
use srgn::actions::German;
use srgn::actions::{
    Action, ActionError, Deletion, Lower, Normalization, Replacement, Style, Titlecase, Upper,
};
#[cfg(feature = "symbols")]
use srgn::actions::{Symbols, SymbolsInversion};
use srgn::iterext::ParallelZipExt;
use srgn::scoping::Scoper;
use srgn::scoping::langs::LanguageScoper;
use srgn::scoping::literal::{Literal, LiteralError};
use srgn::scoping::regex::{Regex, RegexError};
use srgn::scoping::view::ScopedViewBuilder;
use tree_sitter::QueryError as TSQueryError;

// We have `LanguageScoper: Scoper`, but we cannot upcast
// (https://github.com/rust-lang/rust/issues/65991), so hack around the limitation
// by providing both.
type ScoperList = Vec<Box<dyn LanguageScoper>>;

#[expect(clippy::too_many_lines)] // Only slightly above.
#[expect(clippy::cognitive_complexity)]
fn main() -> Result<()> {
    let args = cli::Args::init();

    let level_filter = level_filter_from_env_and_verbosity(args.options.additional_verbosity);
    env_logger::Builder::new()
        .filter_level(level_filter)
        .format_timestamp_micros() // High precision is nice for benchmarks
        .init();

    info!("Launching app with args: {:?}", args);

    let cli::Args {
        scope,
        shell,
        composable_actions,
        standalone_actions,
        mut options,
        languages_scopes,
        #[cfg(feature = "german")]
        german_options,
    } = args;

    if let Some(shell) = shell {
        debug!("Generating completions file for {shell:?}.");
        cli::print_completions(shell, &mut cli::Args::command());
        debug!("Done generating completions file, exiting.");

        return Ok(());
    }

    let standalone_action = standalone_actions.into();

    debug!("Assembling scopers.");
    let general_scoper = get_general_scoper(&options, scope)?;
    // Will be sent across threads and might (the borrow checker is convinced at least)
    // outlive the main one. Scoped threads would work here, `ignore` uses them
    // internally even, but we have no access here.

    let language_scopers = languages_scopes
        .compile_query_sources_to_scopes()?
        .map(Arc::new);
    debug!("Done assembling scopers.");

    let mut actions = {
        debug!("Assembling actions.");
        let mut actions = assemble_common_actions(&composable_actions, standalone_action)?;

        #[cfg(feature = "symbols")]
        if composable_actions.symbols {
            if options.invert {
                actions.push(Box::<SymbolsInversion>::default());
                debug!("Loaded action: SymbolsInversion");
            } else {
                actions.push(Box::<Symbols>::default());
                debug!("Loaded action: Symbols");
            }
        }

        #[cfg(feature = "german")]
        if composable_actions.german {
            actions.push(Box::new(German::new(
                // Smell? Bug if bools swapped.
                german_options.german_prefer_original,
                german_options.german_naive,
            )));
            debug!("Loaded action: German");
        }

        debug!("Done assembling actions.");
        actions
    };

    let is_readable_stdin = grep_cli::is_readable_stdin();
    info!("Detected stdin as readable: {is_readable_stdin}.");

    // See where we're reading from
    let input = match (
        options.stdin_override_to.unwrap_or(is_readable_stdin),
        options.glob.clone(),
        &language_scopers,
    ) {
        // stdin considered viable: always use it.
        (true, None, _)
        // Nothing explicitly available: this should open an interactive stdin prompt.
        | (false, None, None) => Input::Stdin,
        (true, Some(..), _) => {
            // Usage error... warn loudly, the user is likely interested.
            error!("Detected stdin, and request for files: will use stdin and ignore files.");
            Input::Stdin
        }

        // When a pattern is specified, it takes precedence.
        (false, Some(pattern), _) => Input::WalkOn(Box::new(move |path| {
            let res = pattern.matches_path(path);
            trace!("Path '{}' matches: {}.", path.display(), res);
            res
        })),

        // If pattern wasn't manually overridden, consult the language scoper itself, if
        // any.
        (false, None, Some(language_scopers)) => {
            let language_scopers = Arc::clone(language_scopers);
            Input::WalkOn(Box::new(move |path| {
                // TODO: perform this work only once (it's super fast but in the hot
                // path).
                let res = language_scopers
                    .iter()
                    .map(|s| s.is_valid_path(path))
                    .all_equal_value()
                    .expect("all language scopers to agree on path validity");

                trace!(
                    "Language scoper considers path '{}' valid: {}",
                    path.display(),
                    res
                );
                res
            }))
        },
    };

    // Only have this kick in if a language scoper is in play; otherwise, we'd just be a
    // poor imitation of ripgrep itself. Plus, this retains the `tr`-like behavior,
    // setting it apart from other utilities.
    let search_mode = actions.is_empty() && language_scopers.is_some() || options.dry_run;

    if search_mode {
        info!("Will use search mode."); // Modelled after ripgrep!

        let style = if options.dry_run {
            Style::green_bold() // "Would change to this", like git diff
        } else {
            Style::red_bold() // "Found!", like ripgrep
        };
        actions.push(Box::new(style));

        options.only_matching = true;
        options.line_numbers = true;
        options.fail_none = true;
    }

    if actions.is_empty() && !search_mode {
        // Also kind of an error users will likely want to know about.
        error!(
            "No actions specified, and not in search mode. Will return input unchanged, if any."
        );
    }

    let pipeline = if options.dry_run {
        let action: Box<dyn Action> = Box::new(Style::red_bold());
        let color_only = vec![action];
        vec![color_only, actions]
    } else {
        vec![actions]
    };

    let pipeline: Vec<&[Box<dyn Action>]> = pipeline.iter().map(Vec::as_slice).collect();
    let language_scopers = language_scopers.unwrap_or_default();

    // Now write out
    match (input, options.sorted) {
        (Input::Stdin, _ /* no effect */) => {
            info!("Will read from stdin and write to stdout, applying actions.");
            handle_actions_on_stdin(
                &options,
                standalone_action,
                general_scoper.as_ref(),
                &language_scopers,
                &pipeline,
            )?;
        }
        (Input::WalkOn(validator), false) => {
            info!("Will walk file tree, applying actions.");
            handle_actions_on_many_files_threaded(
                &options,
                standalone_action,
                &validator,
                general_scoper.as_ref(),
                &language_scopers,
                &pipeline,
                search_mode,
                options.threads.map_or_else(
                    || std::thread::available_parallelism().map_or(1, std::num::NonZero::get),
                    std::num::NonZero::get,
                ),
            )?;
        }
        (Input::WalkOn(validator), true) => {
            info!("Will walk file tree, applying actions.");
            handle_actions_on_many_files_sorted(
                &options,
                standalone_action,
                &validator,
                general_scoper.as_ref(),
                &language_scopers,
                &pipeline,
                search_mode,
            )?;
        }
    };

    info!("Done, exiting");
    Ok(())
}

/// Indicates whether a filesystem path is valid according to some criteria (glob
/// pattern, ...).
type Validator = Box<dyn Fn(&Path) -> bool + Send + Sync>;

/// The input to read from.
enum Input {
    /// Standard input.
    Stdin,
    /// Use a recursive directory walker, and apply the contained validator, which
    /// indicates valid filesystem entries. This is similar to globbing, but more
    /// flexible.
    WalkOn(Validator),
}

/// A standalone action to perform on the results of applying a scope.
#[derive(Clone, Copy, Debug)]
enum StandaloneAction {
    /// Delete anything in scope.
    ///
    /// Cannot be used with any other action: there is no point in deleting and
    /// performing any other processing. Sibling actions would either receive empty
    /// input or have their work wiped.
    Delete,
    /// Squeeze consecutive occurrences of scope into one.
    Squeeze,
    /// No stand alone action is set.
    None,
}

/// A "pipeline" in that there's not just a single sequence (== slice) of actions, but
/// instead multiple. These can be used in parallel (on the same or different views),
/// and the different results then used for advanced use cases. For example, diffing
/// different results against one another.
type Pipeline<'a> = &'a [&'a [Box<dyn Action>]];

/// Main entrypoint for simple `stdin` -> `stdout` processing.
fn handle_actions_on_stdin(
    global_options: &cli::GlobalOptions,
    standalone_action: StandaloneAction,
    general_scoper: &dyn Scoper,
    language_scopers: &[Box<dyn LanguageScoper>],
    pipeline: Pipeline<'_>,
) -> Result<(), ProgramError> {
    info!("Will use stdin to stdout.");
    let mut source = String::new();
    io::stdin().lock().read_to_string(&mut source)?;
    let mut destination = String::with_capacity(source.len());

    apply(
        global_options,
        standalone_action,
        &source,
        &mut destination,
        general_scoper,
        language_scopers,
        pipeline,
    )?;

    stdout().lock().write_all(destination.as_bytes())?;

    Ok(())
}

/// Main entrypoint for processing using strictly sequential, *single-threaded*
/// processing.
///
/// If it's good enough for [ripgrep], it's good enough for us :-). Main benefit it full
/// control of output for testing anyway.
///
/// [ripgrep]:
///     https://github.com/BurntSushi/ripgrep/blob/71d71d2d98964653cdfcfa315802f518664759d7/GUIDE.md#L1016-L1017
fn handle_actions_on_many_files_sorted(
    global_options: &cli::GlobalOptions,
    standalone_action: StandaloneAction,
    validator: &Validator,
    general_scoper: &dyn Scoper,
    language_scopers: &[Box<dyn LanguageScoper>],
    pipeline: Pipeline<'_>,
    search_mode: bool,
) -> Result<(), ProgramError> {
    let root = env::current_dir()?;
    info!(
        "Will walk file tree sequentially, in sorted order, starting from: {:?}",
        root.canonicalize()
    );

    let mut n_files_processed: usize = 0;
    let mut n_files_seen: usize = 0;
    for entry in WalkBuilder::new(&root)
        .hidden(!global_options.hidden)
        .git_ignore(!global_options.gitignored)
        .sort_by_file_path(Ord::cmp)
        .build()
    {
        match entry {
            Ok(entry) => {
                let path = entry.path();
                let res = process_path(
                    global_options,
                    standalone_action,
                    path,
                    &root,
                    validator,
                    general_scoper,
                    language_scopers,
                    pipeline,
                    search_mode,
                );

                n_files_seen += match res {
                    Err(PathProcessingError::NotAFile | PathProcessingError::InvalidFile) => 0,
                    _ => 1,
                };

                n_files_processed += match res {
                    Ok(()) => 1,

                    // Soft errors with reasonable handling available:
                    Err(PathProcessingError::NotAFile | PathProcessingError::InvalidFile) => 0,
                    Err(PathProcessingError::ApplicationError(ApplicationError::SomeInScope))
                        if global_options.fail_any =>
                    {
                        // Early-out
                        info!("Match at {}, exiting early", path.display());
                        return Err(ProgramError::SomethingProcessed);
                    }
                    #[expect(clippy::match_same_arms)]
                    Err(PathProcessingError::ApplicationError(
                        ApplicationError::NoneInScope | ApplicationError::SomeInScope,
                    )) => 0,
                    Err(PathProcessingError::IoError(e, _))
                        if e.kind() == io::ErrorKind::BrokenPipe && search_mode =>
                    {
                        trace!("Detected broken pipe, stopping search.");
                        break;
                    }
                    Err(PathProcessingError::IoError(e, _))
                        // `InvalidData` does NOT equal "invalid utf-8", but that's how
                        // it's _effectively_ used in the "read to string" type of
                        // functions we use throughout.
                        // https://github.com/rust-lang/rust/blob/096277e989d6de11c3077472fc05778e261e7b8e/library/std/src/io/error.rs#L78-L79
                        if e.kind() == io::ErrorKind::InvalidData =>
                    {
                        warn!("File contains unreadable data (binary? invalid utf-8?), skipped: {}", path.display());
                        0
                    }

                    // Hard errors we should do something about:
                    Err(
                        e @ (PathProcessingError::ApplicationError(ApplicationError::ActionError(
                            ..,
                        ))
                        | PathProcessingError::IoError(..)),
                    ) => {
                        if search_mode {
                            error!("Error walking at {}: {}", path.display(), e);
                            0
                        } else {
                            error!("Aborting walk at {} due to: {}", path.display(), e);
                            return Err(e.into());
                        }
                    }
                }
            }
            Err(e) => {
                if search_mode {
                    error!("Error walking: {}", e);
                } else {
                    error!("Aborting walk due to: {}", e);
                    return Err(e.into());
                }
            }
        }
    }

    info!("Saw {} items", n_files_seen);
    info!("Processed {} files", n_files_processed);

    if n_files_seen == 0 && global_options.fail_no_files {
        Err(ProgramError::NoFilesFound)
    } else if n_files_processed == 0 && global_options.fail_none {
        Err(ProgramError::NothingProcessed)
    } else {
        Ok(())
    }
}

/// Main entrypoint for processing using at least 1 thread.
#[expect(clippy::too_many_lines)]
#[expect(clippy::too_many_arguments)]
fn handle_actions_on_many_files_threaded(
    global_options: &cli::GlobalOptions,
    standalone_action: StandaloneAction,
    validator: &Validator,
    general_scoper: &dyn Scoper,
    language_scopers: &[Box<dyn LanguageScoper>],
    pipeline: Pipeline<'_>,
    search_mode: bool,
    n_threads: usize,
) -> Result<(), ProgramError> {
    let root = env::current_dir()?;
    info!(
        "Will walk file tree using {:?} thread(s), processing in arbitrary order, starting from: {:?}",
        n_threads,
        root.canonicalize()
    );

    let n_files_processed = Arc::new(Mutex::new(0usize));
    let n_files_seen = Arc::new(Mutex::new(0usize));
    let err: Arc<Mutex<Option<ProgramError>>> = Arc::new(Mutex::new(None));

    WalkBuilder::new(&root)
        .threads(
            // https://github.com/BurntSushi/ripgrep/issues/2854
            n_threads,
        )
        .hidden(!global_options.hidden)
        .git_ignore(!global_options.gitignored)
        .build_parallel()
        .run(|| {
            Box::new(|entry| match entry {
                Ok(entry) => {
                    let path = entry.path();
                    let res = process_path(
                        global_options,
                        standalone_action,
                        path,
                        &root,
                        validator,
                        general_scoper,
                        language_scopers,
                        pipeline,
                        search_mode,
                    );

                    match res {
                        Err(PathProcessingError::NotAFile | PathProcessingError::InvalidFile) => (),
                        _ => *n_files_seen.lock().unwrap() += 1,
                    }

                    match res {
                        Ok(()) => {
                            *n_files_processed.lock().unwrap() += 1;
                            WalkState::Continue
                        }

                        // Soft errors with reasonable handling available:
                        Err(PathProcessingError::NotAFile | PathProcessingError::InvalidFile) => {
                            WalkState::Continue
                        }
                        Err(
                            e
                            @ PathProcessingError::ApplicationError(ApplicationError::SomeInScope),
                        ) if global_options.fail_any => {
                            // Early-out
                            info!("Match at {}, exiting early", path.display());
                            *err.lock().unwrap() = Some(e.into());
                            WalkState::Quit
                        }
                        Err(PathProcessingError::ApplicationError(
                            ApplicationError::NoneInScope | ApplicationError::SomeInScope,
                        )) => WalkState::Continue,
                        Err(PathProcessingError::IoError(e, _))
                            if e.kind() == io::ErrorKind::BrokenPipe && search_mode =>
                        {
                            trace!("Detected broken pipe, stopping search.");
                            WalkState::Quit
                        }
                        Err(PathProcessingError::IoError(e, _))
                            // `InvalidData` does NOT equal "invalid utf-8", but that's
                            // how it's _effectively_ used in the "read to string" type
                            // of functions we use throughout.
                            // https://github.com/rust-lang/rust/blob/096277e989d6de11c3077472fc05778e261e7b8e/library/std/src/io/error.rs#L78-L79
                            if e.kind() == io::ErrorKind::InvalidData =>
                        {
                            warn!("File contains unreadable data (binary? invalid utf-8?), skipped: {}", path.display());
                            WalkState::Continue
                        }

                        // Hard errors we should do something about:
                        Err(
                            e @ (PathProcessingError::ApplicationError(..)
                            | PathProcessingError::IoError(..)),
                        ) => {
                            error!("Error walking at {} due to: {}", path.display(), e);

                            if search_mode {
                                WalkState::Continue
                            } else {
                                // Chances are something bad and/or unintended happened;
                                // bail out to limit any potential damage.
                                error!("Aborting walk for safety");
                                *err.lock().unwrap() = Some(e.into());
                                WalkState::Quit
                            }
                        }
                    }
                }
                Err(e) => {
                    if search_mode {
                        error!("Error walking: {}", e);
                        WalkState::Continue
                    } else {
                        error!("Aborting walk due to: {}", e);
                        *err.lock().unwrap() = Some(e.into());
                        WalkState::Quit
                    }
                }
            })
        });

    let error = err.lock().unwrap().take();
    if let Some(e) = error {
        return Err(e);
    }

    let n_files_seen = *n_files_seen.lock().unwrap();
    info!("Saw {} items", n_files_seen);
    let n_files_processed = *n_files_processed.lock().unwrap();
    info!("Processed {} files", n_files_processed);

    if n_files_seen == 0 && global_options.fail_no_files {
        Err(ProgramError::NoFilesFound)
    } else if n_files_processed == 0 && global_options.fail_none {
        Err(ProgramError::NothingProcessed)
    } else {
        Ok(())
    }
}

#[expect(clippy::too_many_arguments)]
fn process_path(
    global_options: &cli::GlobalOptions,
    standalone_action: StandaloneAction,
    path: &Path,
    root: &Path,
    validator: &Validator,
    general_scoper: &dyn Scoper,
    language_scopers: &[Box<dyn LanguageScoper>],
    pipeline: Pipeline<'_>,
    search_mode: bool,
) -> std::result::Result<(), PathProcessingError> {
    if !path.is_file() {
        trace!("Skipping path (not a file): {:?}", path);
        return Err(PathProcessingError::NotAFile);
    }

    let path = diff_paths(path, root).expect("started walk at root, so relative to root works");

    if !validator(&path) {
        trace!("Skipping path (invalid): {:?}", path);
        return Err(PathProcessingError::InvalidFile);
    }

    debug!("Processing path: {:?}", path);

    let (new_contents, filesize, changed) = {
        let mut file = File::open(&path)?;

        let filesize = file.metadata().map_or(0, |m| m.len());
        let mut source =
            String::with_capacity(filesize.try_into().unwrap_or(/* no perf gains for you */ 0));
        file.read_to_string(&mut source)?;

        let mut destination = String::with_capacity(source.len());

        let changed = apply(
            global_options,
            standalone_action,
            &source,
            &mut destination,
            general_scoper,
            language_scopers,
            pipeline,
        )?;

        (destination, filesize, changed)
    };

    // Hold the lock so results aren't intertwined
    let mut stdout = stdout().lock();

    if search_mode {
        if !new_contents.is_empty() {
            writeln!(
                stdout,
                "{}\n{}",
                path.display().to_string().magenta(),
                &new_contents
            )?;
        }
    } else {
        if filesize > 0 && new_contents.is_empty() {
            error!(
                "Failsafe triggered: file {} is nonempty ({} bytes), but new contents are empty. Will not wipe file.",
                path.display(),
                filesize
            );
            return Err(io::Error::new(
                io::ErrorKind::Other,
                "attempt to wipe non-empty file (failsafe guard)",
            )
            .into());
        }

        if changed {
            debug!("Got new file contents, writing to file: {:?}", path);
            assert!(
                !global_options.dry_run,
                // Dry run leverages search mode, so should never get here. Assert for
                // extra safety.
                "Dry running, but attempted to write file!"
            );
            fs::write(&path, new_contents.as_bytes())?;

            // Confirm after successful processing.
            writeln!(stdout, "{}", path.display())?;
        } else {
            debug!(
                "Skipping writing file anew (nothing changed): {}",
                path.display()
            );
        }

        debug!("Done processing file: {:?}", path);
    };

    Ok(())
}

/// Runs the actual core processing, returning whether anything changed in the output
/// compared to the input.
///
/// TODO: The way this interacts with [`process_path`] etc. is just **awful** spaghetti
/// of the most imperative, procedural kind. Refactor needed.
fn apply(
    global_options: &cli::GlobalOptions,
    standalone_action: StandaloneAction,
    source: &str,
    // Use a string to avoid repeated and unnecessary bytes -> utf8 conversions and
    // corresponding checks.
    destination: &mut String,
    general_scoper: &dyn Scoper,
    language_scopers: &[Box<dyn LanguageScoper>],
    pipeline: Pipeline<'_>,
) -> std::result::Result<bool, ApplicationError> {
    debug!("Building view.");
    let mut builder = ScopedViewBuilder::new(source);

    if global_options.join_language_scopes {
        // All at once, as a slice: hits a specific, 'joining' `impl`
        builder.explode(&language_scopers);
    } else {
        // One by one: hits a different, 'intersecting' `impl`
        for scoper in language_scopers {
            builder.explode(scoper);
        }
    }

    builder.explode(general_scoper);
    let mut view = builder.build();
    debug!("Done building view: {view:?}");

    if global_options.fail_none && !view.has_any_in_scope() {
        return Err(ApplicationError::NoneInScope);
    }

    if global_options.fail_any && view.has_any_in_scope() {
        return Err(ApplicationError::SomeInScope);
    };

    debug!("Applying actions to view.");
    if matches!(standalone_action, StandaloneAction::Squeeze) {
        view.squeeze();
    }

    // Give each pipeline its own fresh view
    let mut views = vec![view; pipeline.len()];

    for (actions, view) in pipeline.iter().zip_eq(&mut views) {
        for action in *actions {
            view.map_with_context(action)?;
        }
    }

    debug!("Writing to destination.");
    let line_based = global_options.only_matching || global_options.line_numbers;
    if line_based {
        let line_based_views = views.iter().map(|v| v.lines().into_iter()).collect_vec();

        for (i, lines) in line_based_views.into_iter().parallel_zip().enumerate() {
            let i = i + 1;
            for line in lines {
                if !global_options.only_matching || line.has_any_in_scope() {
                    if global_options.line_numbers {
                        destination.push_str(&format!("{}:", i.to_string().green()));
                    }

                    destination.push_str(&line.to_string());
                }
            }
        }
    } else {
        assert_eq!(
            views.len(),
            1,
            // Multiple views are useful for e.g. diffing, which works line-based (see
            // `dry_run`). When not line-based, they *currently* do not make sense, as
            // there's neither any code path where there *would* be multiple views at
            // this point, *nor* a valid use case. Printing multiple views here would
            // probably wreak havoc.
            "Multiple views at this stage make no sense."
        );

        for view in views {
            destination.push_str(&view.to_string());
        }
    };
    debug!("Done writing to destination.");

    Ok(source != *destination)
}

/// Top-level, user-facing errors, affecting and possibly terminating program execution
/// as a whole.
#[derive(Debug)]
enum ProgramError {
    /// Error when handling a path.
    PathProcessingError(PathProcessingError),
    /// Error when applying.
    ApplicationError(ApplicationError),
    /// No files were found, unexpectedly.
    NoFilesFound,
    /// Files were found but nothing ended up being processed, unexpectedly.
    NothingProcessed,
    /// Files were found but some input ended up being processed, unexpectedly.
    SomethingProcessed,
    /// I/O error.
    IoError(io::Error),
    /// Error while processing files for walking.
    IgnoreError(ignore::Error),
    /// The given query failed to parse
    QueryError(TSQueryError),
}

impl fmt::Display for ProgramError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::PathProcessingError(e) => write!(f, "Error processing path: {e}"),
            Self::ApplicationError(e) => write!(f, "Error applying: {e}"),
            Self::NoFilesFound => write!(f, "No files found"),
            Self::NothingProcessed => write!(f, "No input was in scope"),
            Self::SomethingProcessed => write!(f, "Some input was in scope"),
            Self::IoError(e) => write!(f, "I/O error: {e}"),
            Self::IgnoreError(e) => write!(f, "Error walking files: {e}"),
            Self::QueryError(e) => {
                write!(f, "Error occurred while creating a tree-sitter query: {e}")
            }
        }
    }
}

impl From<ApplicationError> for ProgramError {
    fn from(err: ApplicationError) -> Self {
        Self::ApplicationError(err)
    }
}

impl From<PathProcessingError> for ProgramError {
    fn from(err: PathProcessingError) -> Self {
        Self::PathProcessingError(err)
    }
}

impl From<io::Error> for ProgramError {
    fn from(err: io::Error) -> Self {
        Self::IoError(err)
    }
}

impl From<ignore::Error> for ProgramError {
    fn from(err: ignore::Error) -> Self {
        Self::IgnoreError(err)
    }
}

impl From<TSQueryError> for ProgramError {
    fn from(err: TSQueryError) -> Self {
        Self::QueryError(err)
    }
}

impl Error for ProgramError {}

/// Errors when applying actions to scoped views.
#[derive(Debug)]
enum ApplicationError {
    /// Something was *unexpectedly* in scope.
    SomeInScope,
    /// Nothing was in scope, *unexpectedly*.
    NoneInScope,
    /// Error with an [`Action`].
    ActionError(ActionError),
}

impl fmt::Display for ApplicationError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::SomeInScope => write!(f, "Some input was in scope"),
            Self::NoneInScope => write!(f, "No input was in scope"),
            Self::ActionError(e) => write!(f, "Error in an action: {e}"),
        }
    }
}

impl From<ActionError> for ApplicationError {
    fn from(err: ActionError) -> Self {
        Self::ActionError(err)
    }
}

impl Error for ApplicationError {}

/// Errors when processing a (file) path.
#[derive(Debug)]
enum PathProcessingError {
    /// I/O error.
    IoError(io::Error, Option<PathBuf>),
    /// Item was not a file (directory, symlink, ...).
    NotAFile,
    /// Item is a file but is unsuitable for processing.
    InvalidFile,
    /// Error when applying.
    ApplicationError(ApplicationError),
}

impl fmt::Display for PathProcessingError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::IoError(e, None) => write!(f, "I/O error: {e}"),
            Self::IoError(e, Some(path)) => write!(f, "I/O error at path {}: {e}", path.display()),
            Self::NotAFile => write!(f, "Item is not a file"),
            Self::InvalidFile => write!(f, "Item is not a valid file"),
            Self::ApplicationError(e) => write!(f, "Error applying: {e}"),
        }
    }
}

impl From<io::Error> for PathProcessingError {
    fn from(err: io::Error) -> Self {
        Self::IoError(err, None)
    }
}

impl From<ApplicationError> for PathProcessingError {
    fn from(err: ApplicationError) -> Self {
        Self::ApplicationError(err)
    }
}

impl Error for PathProcessingError {}

#[derive(Debug)]
enum ScoperBuildError {
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
            Self::RegexError(e) => write!(f, "Regex error: {e}"),
            Self::LiteralError(e) => write!(f, "Literal error: {e}"),
        }
    }
}

impl Error for ScoperBuildError {}

fn get_general_scoper(options: &cli::GlobalOptions, scope: String) -> Result<Box<dyn Scoper>> {
    Ok(if options.literal_string {
        Box::new(Literal::try_from(scope).context("Failed building literal string")?)
    } else {
        Box::new(Regex::try_from(scope).context("Failed building regex")?)
    })
}

fn assemble_common_actions(
    composable_actions: &cli::ComposableActions,
    standalone_actions: StandaloneAction,
) -> Result<Vec<Box<dyn Action>>> {
    let mut actions: Vec<Box<dyn Action>> = Vec::new();

    if let Some(replacement) = composable_actions.replace.clone() {
        actions.push(Box::new(
            Replacement::try_from(replacement).context("Failed building replacement string")?,
        ));
        debug!("Loaded action: Replacement");
    }

    if matches!(standalone_actions, StandaloneAction::Delete) {
        actions.push(Box::<Deletion>::default());
        debug!("Loaded action: Deletion");
    }

    if composable_actions.upper {
        actions.push(Box::<Upper>::default());
        debug!("Loaded action: Upper");
    }

    if composable_actions.lower {
        actions.push(Box::<Lower>::default());
        debug!("Loaded action: Lower");
    }

    if composable_actions.titlecase {
        actions.push(Box::<Titlecase>::default());
        debug!("Loaded action: Titlecase");
    }

    if composable_actions.normalize {
        actions.push(Box::<Normalization>::default());
        debug!("Loaded action: Normalization");
    }

    Ok(actions)
}

/// To the default log level found in the environment, adds the requested additional
/// verbosity level, clamped to the maximum available.
///
/// See also
/// <https://docs.rs/env_logger/latest/env_logger/struct.Env.html#default-environment-variables>
/// and <https://docs.rs/env_logger/latest/env_logger/#enabling-logging>
fn level_filter_from_env_and_verbosity(additional_verbosity: u8) -> LevelFilter {
    let available = LevelFilter::iter().collect::<Vec<_>>();
    let default = env_logger::Builder::from_default_env().build().filter();

    let mut level = default as usize; // Implementation detail of `log` crate
    level += additional_verbosity as usize;

    available.get(level).copied().unwrap_or_else(|| {
        eprintln!(
            "Requested additional verbosity on top of env default exceeds maximum, will use maximum"
        );

        available
            .last()
            .copied()
            .expect("At least one level must be available")
    })
}

mod cli {
    use std::num::NonZero;
    use std::path::PathBuf;
    use std::{fs, io};

    use clap::builder::ArgPredicate;
    use clap::{ArgAction, Command, CommandFactory, Parser};
    use clap_complete::{Generator, Shell, generate};
    use log::info;
    use srgn::GLOBAL_SCOPE;
    use srgn::scoping::langs::{
        LanguageScoper, QuerySource, c, csharp, go, hcl, python, rust, typescript,
    };
    use tree_sitter::QueryError as TSQueryError;

    use crate::{ProgramError, StandaloneAction};

    /// Main CLI entrypoint.
    ///
    /// Using `verbatim_doc_comment` a lot as otherwise lines wouldn't wrap neatly. I
    /// format them narrowly manually anyway, so can just use them verbatim.
    #[derive(Parser, Debug)]
    #[command(
        author,
        version,
        about,
        long_about = None,
        // Really dumb to hard-code, but we need deterministic output for README tests
        // to remain stable, and this is probably both a solid default *and* plays with
        // this very source file which is wrapped at *below* that, so it fits and clap
        // doesn't touch our manually formatted doc strings anymore.
        term_width = 90,
    )]
    pub struct Args {
        /// Scope to apply to, as a regular expression pattern.
        ///
        /// If string literal mode is requested, will be interpreted as a literal
        /// string.
        ///
        /// Actions will apply their transformations within this scope only.
        ///
        /// The default is the global scope, matching the entire input. Where that
        /// default is meaningless or dangerous (e.g., deletion), this argument is
        /// required.
        #[arg(
            value_name = "SCOPE",
            default_value = GLOBAL_SCOPE,
            verbatim_doc_comment,
            default_value_if("literal_string", ArgPredicate::IsPresent, None)
        )]
        pub(super) scope: String,

        /// Print shell completions for the given shell.
        #[arg(long = "completions", value_enum, verbatim_doc_comment)]
        // This thing needs to live up here to show up within `Options` next to `--help`
        // and `--version`. Further down, it'd show up in the wrong section because we
        // alter `next_help_heading`.
        pub(super) shell: Option<Shell>,

        #[command(flatten)]
        pub(super) composable_actions: ComposableActions,

        #[command(flatten)]
        pub(super) standalone_actions: StandaloneActions,

        #[command(flatten)]
        pub(super) options: GlobalOptions,

        #[command(flatten)]
        pub(super) languages_scopes: LanguageScopes,

        #[cfg(feature = "german")]
        #[command(flatten)]
        pub(super) german_options: GermanOptions,
    }

    /// <https://github.com/clap-rs/clap/blob/f65d421607ba16c3175ffe76a20820f123b6c4cb/clap_complete/examples/completion-derive.rs#L69>
    pub fn print_completions<G: Generator>(generator: G, cmd: &mut Command) {
        generate(
            generator,
            cmd,
            cmd.get_name().to_string(),
            &mut io::stdout(),
        );
    }

    #[derive(Parser, Debug)]
    #[group(required = false, multiple = true)]
    #[command(next_help_heading = "Options (global)")]
    #[expect(clippy::struct_excessive_bools)]
    pub struct GlobalOptions {
        /// Glob of files to work on (instead of reading stdin).
        ///
        /// If actions are applied, they overwrite files in-place.
        ///
        /// For supported glob syntax, see:
        /// <https://docs.rs/glob/0.3.1/glob/struct.Pattern.html>
        ///
        /// Names of processed files are written to stdout.
        #[arg(short('G'), long, verbatim_doc_comment, alias = "files")]
        pub glob: Option<glob::Pattern>,
        /// Fail if working on files (e.g. globbing is requested) but none are found.
        ///
        /// Processing no files is not an error condition in itself, but might be an
        /// unexpected outcome in some contexts. This flag makes the condition explicit.
        #[arg(long, verbatim_doc_comment, alias = "fail-empty-glob")]
        pub fail_no_files: bool,
        /// Do not destructively overwrite files, instead print rich diff only.
        ///
        /// The diff details the names of files which would be modified, alongside all
        /// changes inside those files which would be performed outside of dry running.
        /// It is similar to git diff with word diffing enabled.
        #[arg(long, verbatim_doc_comment)]
        pub dry_run: bool,
        /// Undo the effects of passed actions, where applicable.
        ///
        /// Requires a 1:1 mapping between replacements and original, which is currently
        /// available only for:
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
        /// normally.
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
        /// Join (logical 'OR') multiple language scopes, instead of intersecting them.
        ///
        /// The default when multiple language scopes are given is to intersect their
        /// scopes, left to right. For example, `--go func --go strings` will first
        /// scope down to `func` bodies, then look for strings only within those. This
        /// flag instead joins (in the set logic sense) all scopes. The example would
        /// then scope any `func` bodies, and any strings, anywhere. Language scopers
        /// can then also be given in any order.
        ///
        /// No effect if only a single language scope is given. Also does not affect
        /// non-language scopers (regex pattern etc.), which always intersect.
        #[arg(short('j'), long, verbatim_doc_comment)]
        pub join_language_scopes: bool,
        /// Prepend line numbers to output.
        #[arg(long, hide(true), verbatim_doc_comment)]
        // Hidden: internal use. Not really useful to expose.
        pub line_numbers: bool,
        /// Print only matching lines.
        #[arg(long, hide(true), verbatim_doc_comment)]
        // Hidden: internal use. Not really useful to expose.
        pub only_matching: bool,
        /// Do not ignore hidden files and directories.
        #[arg(short('H'), long, verbatim_doc_comment)]
        pub hidden: bool,
        /// Do not ignore `.gitignore`d files and directories.
        #[arg(long, verbatim_doc_comment)]
        pub gitignored: bool,
        /// Process files in lexicographically sorted order, by file path.
        ///
        /// In search mode, this emits results in sorted order. Otherwise, it processes
        /// files in sorted order.
        ///
        /// Sorted processing disables parallel processing.
        #[arg(long, verbatim_doc_comment)]
        pub sorted: bool,
        /// Override detection heuristics for stdin readability, and force to value.
        ///
        /// `true` will always attempt to read from stdin. `false` will never read from
        /// stdin, even if provided.
        #[arg(long, hide(true), verbatim_doc_comment)]
        // Hidden: internal use for testing, where some forceful overriding is required.
        pub stdin_override_to: Option<bool>,
        /// Number of threads to run processing on, when working with files.
        ///
        /// If not specified, will default to available parallelism. Set to 1 for
        /// sequential, deterministic (but not sorted) output.
        #[arg(long, verbatim_doc_comment)]
        pub threads: Option<NonZero<usize>>,
        /// Increase log verbosity level.
        ///
        /// The base log level to use is read from the `RUST_LOG` environment variable
        /// (if unspecified, defaults to 'error'), and increased according to the number
        /// of times this flag is given, maxing out at 'trace' verbosity.
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
    #[expect(clippy::struct_excessive_bools)]
    pub struct ComposableActions {
        /// Replace anything in scope with this value.
        ///
        /// Variables are supported: if a regex pattern was used for scoping and
        /// captured content in named or numbered capture groups, access these in the
        /// replacement value using `$1` etc. for numbered, `$NAME` etc. for named
        /// capture groups.
        ///
        /// This action is specially treated as a positional argument for ergonomics and
        /// compatibility with `tr`.
        ///
        /// If given, will run before any other action.
        #[arg(value_name = "REPLACEMENT", env, verbatim_doc_comment)]
        pub replace: Option<String>,
        /// Uppercase anything in scope.
        #[arg(short, long, env, verbatim_doc_comment)]
        pub upper: bool,
        /// Lowercase anything in scope.
        #[arg(short, long, env, verbatim_doc_comment)]
        pub lower: bool,
        /// Titlecase anything in scope.
        #[arg(short, long, env, verbatim_doc_comment)]
        pub titlecase: bool,
        /// Normalize (Normalization Form D) anything in scope, and throw away marks.
        #[arg(short, long, env, verbatim_doc_comment)]
        pub normalize: bool,
        /// Perform substitutions on German words, such as 'Abenteuergruesse' to
        /// 'Abenteuergrüße', for anything in scope.
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
        /// Perform substitutions on symbols, such as '!=' to '≠', '->' to '→', on
        /// anything in scope.
        ///
        /// Helps translate 'ASCII art' into native Unicode representations.
        #[cfg(feature = "symbols")]
        #[arg(short = 'S', long, verbatim_doc_comment)]
        pub symbols: bool,
    }

    #[derive(Parser, Debug)]
    #[group(required = false, multiple = false)]
    #[command(next_help_heading = "Standalone Actions (only usable alone)")]
    pub struct StandaloneActions {
        /// Delete anything in scope.
        ///
        /// Cannot be used with any other action: there is no point in deleting and
        /// performing any other processing. Sibling actions would either receive empty
        /// input or have their work wiped.
        #[arg(
            short,
            long,
            requires = "scope",
            conflicts_with = stringify!(ComposableActions),
            verbatim_doc_comment
        )]
        pub delete: bool,
        /// Squeeze consecutive occurrences of scope into one.
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

    impl From<StandaloneActions> for StandaloneAction {
        fn from(value: StandaloneActions) -> Self {
            if value.delete {
                Self::Delete
            } else if value.squeeze {
                Self::Squeeze
            } else {
                Self::None
            }
        }
    }

    /// For use as <https://docs.rs/clap/latest/clap/struct.Arg.html#method.value_name>
    const TREE_SITTER_QUERY_VALUE: &str = "TREE-SITTER-QUERY-VALUE";
    const TREE_SITTER_QUERY_FILENAME: &str = "TREE-SITTER-QUERY-FILENAME";

    macro_rules! impl_lang_scopes {
        ($(($lang_flag:ident, $lang_query_flag:ident, $lang_query_file_flag:ident, $lang_scope:ident),)+) => {
            #[derive(Parser, Debug)]
            #[group(required = false, multiple = false)]
            #[command(next_help_heading = "Language scopes")]
            pub struct LanguageScopes {
                $(
                    #[command(flatten)]
                    $lang_flag: Option<$lang_scope>,
                )+
            }

            impl LanguageScopes {
                /// Finds the first language field set, if any, and compiles the `QuerySourceOrPath`'s into a list of `LanguageScoper`'s.
                pub(super) fn compile_query_sources_to_scopes(self) -> Result<Option<crate::ScoperList>, ProgramError> {
                    assert_exclusive_lang_scope(&[
                        $(self.$lang_flag.is_some(),)+
                    ]);

                    $(
                        if let Some(s) = self.$lang_flag {
                            let s = accumulate_scopes::<$lang_flag::CompiledQuery, _>(s.$lang_flag, s.$lang_query_flag, s.$lang_query_file_flag,)?;
                            return Ok(Some(s));
                        }
                    )+

                    Ok(None)
                }
            }
        };
    }

    impl_lang_scopes!(
        (c, c_query, c_query_file, CScope),
        (csharp, csharp_query, csharp_query_file, CSharpScope),
        (go, go_query, go_query_file, GoScope),
        (hcl, hcl_query, hcl_query_file, HclScope),
        (python, python_query, python_query_file, PythonScope),
        (rust, rust_query, rust_query_file, RustScope),
        (
            typescript,
            typescript_query,
            typescript_query_file,
            TypeScriptScope
        ),
    );

    /// Assert that either zero or one lang field is set.
    ///
    /// If the assertion fails, exit with an error message.
    fn assert_exclusive_lang_scope(fields_set: &[bool]) {
        let set_fields_count = fields_set.iter().filter(|b| **b).count();

        if set_fields_count > 1 {
            let mut cmd = Args::command();
            cmd.error(
                clap::error::ErrorKind::ArgumentConflict,
                "Can only use one language at a time.",
            )
            .exit();
        }
    }

    /// Convert the prepared queries and the literal queries into `CompiledQuery`'s
    fn accumulate_scopes<CQ, PQ>(
        prepared_queries: Vec<PQ>,
        literal_queries: Vec<QueryLiteral>,
        file_queries: Vec<PathBuf>,
    ) -> Result<super::ScoperList, ProgramError>
    where
        CQ: LanguageScoper + TryFrom<QuerySource, Error = TSQueryError> + 'static,
        PQ: Into<CQ>,
    {
        let mut scopers: crate::ScoperList = Vec::new();

        for prepared_query in prepared_queries {
            let compiled_query = prepared_query.into();
            scopers.push(Box::new(compiled_query));
        }

        for query_literal in literal_queries {
            let query_source = query_literal.into();
            let compiled_query = CQ::try_from(query_source)?;
            scopers.push(Box::new(compiled_query));
        }

        for file_query in file_queries {
            let query_source = read_query_from_file(file_query)?;
            let compiled_query = CQ::try_from(query_source)?;
            scopers.push(Box::new(compiled_query));
        }

        Ok(scopers)
    }

    /// Read a literal query as a file.
    fn read_query_from_file(path: PathBuf) -> io::Result<QuerySource> {
        info!("Reading query from file at '{}'", path.display());
        let s = fs::read_to_string(path)?;
        Ok(QuerySource::from(s))
    }

    #[derive(Parser, Debug, Clone)]
    #[group(required = false, multiple = false)]
    struct CScope {
        /// Scope C code using a prepared query.
        #[arg(long, env, verbatim_doc_comment)]
        c: Vec<c::PreparedQuery>,

        /// Scope C code using a custom tree-sitter query.
        #[arg(long, env, verbatim_doc_comment, value_name = TREE_SITTER_QUERY_VALUE)]
        c_query: Vec<QueryLiteral>,

        /// Scope C code using a custom tree-sitter query from file.
        #[arg(long, env, verbatim_doc_comment, value_name = TREE_SITTER_QUERY_FILENAME)]
        c_query_file: Vec<PathBuf>,
    }

    #[derive(Parser, Debug, Clone)]
    #[group(required = false, multiple = false)]
    struct CSharpScope {
        /// Scope C# code using a prepared query.
        #[arg(long, env, verbatim_doc_comment, visible_alias = "cs")]
        csharp: Vec<csharp::PreparedQuery>,

        /// Scope C# code using a custom tree-sitter query.
        #[arg(long, env, verbatim_doc_comment, value_name = TREE_SITTER_QUERY_VALUE)]
        csharp_query: Vec<QueryLiteral>,

        /// Scope C# code using a custom tree-sitter query from file.
        #[arg(long, env, verbatim_doc_comment, value_name = TREE_SITTER_QUERY_FILENAME)]
        csharp_query_file: Vec<PathBuf>,
    }

    #[derive(Parser, Debug, Clone)]
    #[group(required = false, multiple = false)]
    struct HclScope {
        #[expect(clippy::doc_markdown)] // CamelCase detected as 'needs backticks'
        /// Scope HashiCorp Configuration Language code using a prepared query.
        #[arg(long, env, verbatim_doc_comment)]
        hcl: Vec<hcl::PreparedQuery>,

        #[expect(clippy::doc_markdown)] // CamelCase detected as 'needs backticks'
        /// Scope HashiCorp Configuration Language code using a custom tree-sitter query.
        #[arg(long, env, verbatim_doc_comment, value_name = TREE_SITTER_QUERY_VALUE)]
        hcl_query: Vec<QueryLiteral>,

        #[expect(clippy::doc_markdown)] // CamelCase detected as 'needs backticks'
        /// Scope HashiCorp Configuration Language code using a custom tree-sitter query
        /// from file.
        #[arg(long, env, verbatim_doc_comment, value_name = TREE_SITTER_QUERY_FILENAME)]
        hcl_query_file: Vec<PathBuf>,
    }

    #[derive(Parser, Debug, Clone)]
    #[group(required = false, multiple = false)]
    struct GoScope {
        /// Scope Go code using a prepared query.
        #[arg(long, env, verbatim_doc_comment)]
        go: Vec<go::PreparedQuery>,

        /// Scope Go code using a custom tree-sitter query.
        #[arg(long, env, verbatim_doc_comment, value_name = TREE_SITTER_QUERY_VALUE)]
        go_query: Vec<QueryLiteral>,

        /// Scope Go code using a custom tree-sitter query from file.
        #[arg(long, env, verbatim_doc_comment, value_name = TREE_SITTER_QUERY_FILENAME)]
        go_query_file: Vec<PathBuf>,
    }

    #[derive(Parser, Debug, Clone)]
    #[group(required = false, multiple = false)]
    struct PythonScope {
        /// Scope Python code using a prepared query.
        #[arg(long, env, verbatim_doc_comment, visible_alias = "py")]
        python: Vec<python::PreparedQuery>,

        /// Scope Python code using a custom tree-sitter query.
        #[arg(long, env, verbatim_doc_comment, value_name = TREE_SITTER_QUERY_VALUE)]
        python_query: Vec<QueryLiteral>,

        /// Scope Python code using a custom tree-sitter query from file.
        #[arg(long, env, verbatim_doc_comment, value_name = TREE_SITTER_QUERY_FILENAME)]
        python_query_file: Vec<PathBuf>,
    }

    #[derive(Parser, Debug, Clone)]
    #[group(required = false, multiple = false)]
    struct RustScope {
        /// Scope Rust code using a prepared query.
        #[arg(long, env, verbatim_doc_comment, visible_alias = "rs")]
        rust: Vec<rust::PreparedQuery>,

        /// Scope Rust code using a custom tree-sitter query.
        #[arg(long, env, verbatim_doc_comment, value_name = TREE_SITTER_QUERY_VALUE)]
        rust_query: Vec<QueryLiteral>,

        /// Scope Rust code using a custom tree-sitter query from file.
        #[arg(long, env, verbatim_doc_comment, value_name = TREE_SITTER_QUERY_FILENAME)]
        rust_query_file: Vec<PathBuf>,
    }

    #[derive(Parser, Debug, Clone)]
    #[group(required = false, multiple = false)]
    struct TypeScriptScope {
        /// Scope TypeScript code using a prepared query.
        #[arg(long, env, verbatim_doc_comment, visible_alias = "ts")]
        typescript: Vec<typescript::PreparedQuery>,

        /// Scope TypeScript code using a custom tree-sitter query.
        #[arg(long, env, verbatim_doc_comment, value_name = TREE_SITTER_QUERY_VALUE)]
        typescript_query: Vec<QueryLiteral>,

        /// Scope TypeScript code using a custom tree-sitter query from file.
        #[arg(long, env, verbatim_doc_comment, value_name = TREE_SITTER_QUERY_FILENAME)]
        typescript_query_file: Vec<PathBuf>,
    }

    #[cfg(feature = "german")]
    #[derive(Parser, Debug)]
    #[group(required = false, multiple = true, id("german-opts"))]
    #[command(next_help_heading = "Options (german)")]
    pub struct GermanOptions {
        /// When some original version and its replacement are equally legal, prefer the
        /// original and do not modify.
        ///
        /// For example, "Busse" (original) and "Buße" (replacement) are equally legal
        /// words: by default, the tool would prefer the latter.
        #[arg(long, env, verbatim_doc_comment)]
        // More fine-grained control is not available. We are not in the business of
        // natural language processing or LLMs, so that's all we can offer...
        pub german_prefer_original: bool,
        /// Always perform any possible replacement ('ae' -> 'ä', 'ss' -> 'ß', etc.),
        /// regardless of legality of the resulting word
        ///
        /// Useful for names, which are otherwise not modifiable as they do not occur in
        /// dictionaries. Called 'naive' as this does not perform legal checks.
        #[arg(long, env, verbatim_doc_comment)]
        pub german_naive: bool,
    }

    /// The literal query as read in from the CLI.
    #[derive(Clone, Debug)]
    struct QueryLiteral(String);

    impl From<String> for QueryLiteral {
        fn from(s: String) -> Self {
            Self(s)
        }
    }

    impl From<QueryLiteral> for QuerySource {
        fn from(query: QueryLiteral) -> Self {
            query.0.into()
        }
    }

    impl Args {
        pub(super) fn init() -> Self {
            Self::parse()
        }

        pub(super) fn command() -> Command {
            <Self as CommandFactory>::command()
        }
    }
}

#[cfg(test)]
mod tests {
    use std::env;
    use std::sync::LazyLock;

    use env_logger::DEFAULT_FILTER_ENV;
    use log::LevelFilter;
    use rstest::rstest;

    use super::*;

    static ENV_MUTEX: LazyLock<Mutex<()>> = LazyLock::new(|| Mutex::new(()));

    #[rstest]
    #[case(None, 0, LevelFilter::Error)]
    #[case(None, 1, LevelFilter::Warn)]
    #[case(None, 2, LevelFilter::Info)]
    #[case(None, 3, LevelFilter::Debug)]
    #[case(None, 4, LevelFilter::Trace)]
    #[case(None, 5, LevelFilter::Trace)]
    #[case(None, 128, LevelFilter::Trace)]
    #[case(Some("off"), 0, LevelFilter::Off)]
    #[case(Some("off"), 1, LevelFilter::Error)]
    #[case(Some("off"), 2, LevelFilter::Warn)]
    #[case(Some("off"), 3, LevelFilter::Info)]
    #[case(Some("off"), 4, LevelFilter::Debug)]
    #[case(Some("off"), 5, LevelFilter::Trace)]
    #[case(Some("off"), 6, LevelFilter::Trace)]
    #[case(Some("off"), 128, LevelFilter::Trace)]
    #[case(Some("error"), 0, LevelFilter::Error)]
    #[case(Some("error"), 1, LevelFilter::Warn)]
    #[case(Some("error"), 2, LevelFilter::Info)]
    #[case(Some("error"), 3, LevelFilter::Debug)]
    #[case(Some("error"), 4, LevelFilter::Trace)]
    #[case(Some("error"), 5, LevelFilter::Trace)]
    #[case(Some("error"), 128, LevelFilter::Trace)]
    #[case(Some("warn"), 0, LevelFilter::Warn)]
    #[case(Some("warn"), 1, LevelFilter::Info)]
    #[case(Some("warn"), 2, LevelFilter::Debug)]
    #[case(Some("warn"), 3, LevelFilter::Trace)]
    #[case(Some("warn"), 4, LevelFilter::Trace)]
    #[case(Some("warn"), 128, LevelFilter::Trace)]
    #[case(Some("info"), 0, LevelFilter::Info)]
    #[case(Some("info"), 1, LevelFilter::Debug)]
    #[case(Some("info"), 2, LevelFilter::Trace)]
    #[case(Some("info"), 3, LevelFilter::Trace)]
    #[case(Some("info"), 128, LevelFilter::Trace)]
    #[case(Some("debug"), 0, LevelFilter::Debug)]
    #[case(Some("debug"), 1, LevelFilter::Trace)]
    #[case(Some("debug"), 2, LevelFilter::Trace)]
    #[case(Some("debug"), 128, LevelFilter::Trace)]
    #[case(Some("trace"), 0, LevelFilter::Trace)]
    #[case(Some("trace"), 1, LevelFilter::Trace)]
    #[case(Some("trace"), 128, LevelFilter::Trace)]
    fn test_level_filter_from_env_and_verbosity(
        #[case] env_value: Option<&str>,
        #[case] additional_verbosity: u8,
        #[case] expected: LevelFilter,
    ) {
        let _guard = ENV_MUTEX.lock().unwrap();

        #[expect(unsafe_code)]
        if let Some(env_value) = env_value {
            unsafe {
                env::set_var(DEFAULT_FILTER_ENV, env_value);
            }
        } else {
            unsafe {
                // Might be set on parent and fork()ed down
                env::remove_var(DEFAULT_FILTER_ENV);
            }
        }

        let result = level_filter_from_env_and_verbosity(additional_verbosity);
        assert_eq!(result, expected);
    }
}
