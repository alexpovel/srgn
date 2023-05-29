use crate::stages::{
    german::{
        machine::{StateMachine, Transition},
        words::{Replace, Replacement, WordCasing},
    },
    tooling::StageResult,
    Stage,
};
use cached::proc_macro::cached;
use cached::SizedCache;
use common::lookup::binary_search_uneven;
use common::strings::is_compound_word;
use itertools::Itertools;
use log::{debug, trace};
use unicode_titlecase::StrTitleCase;

static VALID_GERMAN_WORDS: &str = include_str!(concat!(env!("OUT_DIR"), "/de.txt")); // Generated in `build.rs`.

/// German language stage, responsible for Umlauts and Eszett.
///
/// This stage is responsible for applying the following rules, **where applicable**:
/// - [*Umlauts*](https://en.wikipedia.org/wiki/Umlaut_(diacritic)): replace `ue`, `oe`,
///   `ae` with `ü`, `ö`, `ä`, respectively,
/// - [*Eszett*](https://en.wikipedia.org/wiki/%C3%9F): replace `ss` with `ß`.
///
/// Mechanisms are in place to uphold the following properties:
/// - both lower- and uppercase variants are handled correctly,
/// - compound words are handled correctly.
///
/// Towards this, this stage does *not* simply replace all occurrences, but performs
/// checks to ensure only valid replacements are made. The core of these checks is an
/// exhaustive word list. The better the word list, the better the results. As such, any
/// errors in processing could be the result of a faulty word list *or* faulty
/// algorithms.
///
/// # Example: A simple greeting, with Umlaut and Eszett
///
/// ```
/// use betterletter::{Stage, stages::GermanStage};
///
/// let result: String = GermanStage.substitute("Gruess Gott!").unwrap().into();
/// assert_eq!(result, "Grüß Gott!");
/// ```
///
/// # Example: A compound word
///
/// Note that this compound word is *not* part of the word list (that would be an
/// *elaborate* word list!), but is still handled, as its constituents are.
///
/// ```
/// use betterletter::{Stage, stages::GermanStage};
///
/// let result: String = GermanStage.substitute("Du Suesswassertagtraeumer!").unwrap().into();
/// assert_eq!(result, "Du Süßwassertagträumer!");
/// ```
///
/// # Example: Words *validly* containing alternative Umlaut spelling
///
/// These spellings are *not* replaced, as they are valid words in their own right.
/// Naive implementations/translations (e.g.
/// [`tr`](https://en.wikipedia.org/wiki/Tr_(Unix))) would not handle this correctly.
///
/// ```
/// use betterletter::{Stage, stages::GermanStage};
///
/// for word in &[
///     // "ae"
///     "Aerodynamik",   // should not be "Ärodynamik"
///     "Israel",        // should not be "Isräl"
///     "Schufaeintrag", // should not be "Schufäintrag"
///     // "oe"
///     "Koeffizient",   // should not be "Köffizient"
///     "Dominoeffekt",  // should not be "Dominöffekt"
///     "Poet",          // should not be "Pöt"
///     // "ue"
///     "Abenteuer",     // should not be "Abenteür"
///     "Mauer",         // should not be "Maür"
///     "Steuerung",     // should not be "Steürung"
/// ] {
///     let result: String = GermanStage.substitute(word).unwrap().into();
///     assert_eq!(result, word.to_string());
/// }
/// ```
///
/// Note that `ss`/`ß` is not mentioned, as it is handled
/// [elsewhere](#example-words-with-valid-alternative-and-special-character-spellings).
///
/// # Example: Words with valid alternative *and* special character spellings
///
/// Some words are validly spelled with alternative Umlauts *and* special characters *in
/// the same position*, such as:
/// - [Ma**ß**e](https://de.wiktionary.org/wiki/Ma%C3%9Fe): pertaining to measurements
/// - [Ma**ss**e](https://de.wiktionary.org/wiki/Masse): pertaining to mass/weight
///
/// So if a user inputs `Masse` (they can't spell `Maße`, else they wouldn't have
/// reached for this crate in the first place), what do they mean? Such cases are
/// tricky, as there isn't an easy solution without reaching for full-blown
/// [NLP](https://en.wikipedia.org/wiki/Natural_language_processing) or ML, as the
/// word's context would be required. This stage is much too limited for that. A choice
/// has to be made:
///
/// - do not replace: keep alternative spelling, or
/// - replace: keep special character spelling.
///
/// This tool chooses the latter, as it seems [the least
/// astonishing](https://en.wikipedia.org/wiki/Principle_of_least_astonishment) in the
/// context of this tool, whose entire point is to **make replacements if they're
/// valid**.
///
/// This is an issue mainly for Eszett (`ß`), as for it, two valid spellings are much
/// more likely than for Umlauts.
///
/// ```
/// use betterletter::{Stage, stages::GermanStage};
///
/// for (input, output) in &[
///     ("Busse", "Buße"), // busses / penance
///     ("Masse", "Maße"), // mass / measurements
/// ] {
///     let result: String = GermanStage.substitute(input).unwrap().into();
///     assert_eq!(result, output.to_string());
/// }
/// ```
///
/// # Example: Upper- and mixed case
///
/// This stage can handle any case, but assumes **nouns are never lower case** (a pretty
/// mild assumption). The **first letter governs the case** of the entity (Umlaut,
/// Eszett or entire word) in question:
///
/// | Input | Example Umlaut/Eszett | Example word | Detected case |
/// | ----- | --------------------- | ------------ | ------------- |
/// | `xx`  | `ue`                  | `hello`      | lowercase     |
/// | `xX`  | `sS`                  | `hElLo`      | lowercase     |
/// | `Xx`  | `Ue`                  | `Hello`      | uppercase     |
/// | `XX`  | `SS`                  | `HELLooo`    | uppercase     |
///
/// The same principle then further applies to entire words, which is especially
/// noticeable for mixed-case ones. The word list is not going to contain mixed-case
/// words, so a decision has to be made: what case will candidates be checked against?
/// If whatever case was detected is not considered a valid word, the replacement is not
/// made. Example flows follow.
///
/// ## Subexample: mixed case, invalid word
///
/// The flow looks like:
///
/// `aEpFeL` → lowercase Umlaut → `äpFeL` → lowercase word → squash → `äpfel` → ❌ →
/// output is `aEpFeL`
///
///
/// ```
/// use betterletter::{Stage, stages::GermanStage};
///
/// let result: String = GermanStage.substitute("aEpFeL").unwrap().into();
///
/// // Error: MiXeD CaSe noun without leading capital letter
/// assert_eq!(result, "aEpFeL");
/// ```
///
/// ## Subexample: mixed case, valid word
///
/// The flow looks like:
///
/// `AePfEl` → uppercase Umlaut → `ÄPfEl` → uppercase word → squash → `Äpfel` → ✅ →
/// output is `Äpfel`
///
/// ```
/// use betterletter::{Stage, stages::GermanStage};
///
/// let result: String = GermanStage.substitute("AePfEl").unwrap().into();
///
/// // OK: MiXeD CaSe words nouns are okay, *if* starting with a capital letter
/// assert_eq!(result, "ÄPfEl");
/// ```
///
/// ## Subexample: other cases
///
/// ```
/// use betterletter::{Stage, stages::GermanStage};
///
/// let f = |word: &str| -> String {GermanStage.substitute(word).unwrap().into()};
///
/// // OK: The normal case, adjective lowercase
/// assert_eq!(f("Voll suess!"), "Voll süß!");
///
/// // OK: Adjective uppercase (start of sentence)
/// assert_eq!(f("Suesses Eis!"), "Süßes Eis!");
///
/// // OK: Uppercased noun
/// assert_eq!(f("Aepfel"), "Äpfel");
///
/// // Error: Lowercased noun is *not* replaced, we are not a spell checker
/// assert_eq!(f("aepfel"), "aepfel");
///
/// // OK: SCREAMING CASE noun is okay though
/// assert_eq!(f("AEPFEL"), "ÄPFEL");
///
/// // OK: SCREAMING CASE verb is okay as well
/// assert_eq!(f("SCHLIESSEN"), "SCHLIEẞEN");
///
/// // OK: MiXeD CaSe verb: inserted special character is uppercase
/// assert_eq!(f("fUeLleN"), "fÜLleN");
///
/// // OK: MiXeD CaSe verb: inserted special character is lowercase
/// assert_eq!(f("FuElLEn"), "FülLEn");
/// ```
///
/// ### Capital Eszett (ẞ)
///
/// Note the spelling of `SCHLIEẞEN` containing `ẞ`, the [uppercase version of
/// `ß`](https://www.wikidata.org/wiki/Q9693), part of [official spelling since
/// 2017](https://web.archive.org/web/20230206102049/https://www.rechtschreibrat.com/DOX/rfdr_PM_2017-06-29_Aktualisierung_Regelwerk.pdf).
/// It's the result of uppercasing `ß` of `schließen`. This does **not** follow Rust's
/// usual behavior, which is why it is specially mentioned here:
///
/// ```
/// let lc = "ß";
/// let uc = "ẞ";
///
/// assert_eq!(lc.to_uppercase().to_string(), "SS");
///
/// // The other way around works though:
/// assert_eq!(uc.to_lowercase().to_string(), lc);
///
/// // Uppercase stays uppercase:
/// assert_eq!(uc.to_uppercase().to_string(), uc);
///
/// // Lowercase stays lowercase (as opposed to `ss`):
/// assert_eq!(lc.to_lowercase().to_string(), lc);
/// ```
///
/// The `SS` of `SCHLIESSEN` is detected as an uppercase Eszett, which is specifically
/// inserted. You might want to run additional processing if this is undesired.
///
/// # Example: Other bytes
///
/// This stage handles the German alphabet *only*, and will leave other input bytes
/// untouched. You get to keep your trailing newlines, emojis (also multi-[`char`] ones),
/// and everything else.
///
/// Of course, the input has to be valid UTF-8, as is ensured by its signature ([`str`]).
///
/// ```
/// use betterletter::{Stage, stages::GermanStage};
///
/// let result: String = GermanStage.substitute("\0Schoener    你好 Satz... 👋🏻\r\n\n").unwrap().into();
/// assert_eq!(result, "\0Schöner    你好 Satz... 👋🏻\r\n\n");
/// ```
///
/// # Performance
///
/// This stage is implemented as a [finite state
/// machine](https://en.wikipedia.org/wiki/Finite-state_machine), which means it runs in
/// linear time as well as constant space. It is therefore very fast and memory
/// efficient, requiring only a single pass over the input [`str`].
///
/// The underlying checks for valid words are implemented as a
/// [memoized](https://en.wikipedia.org/wiki/Memoization), recursive binary search.
/// While they're fast, other methods could be faster but weren't chosen for one or more
/// of these reasons:
///
/// - poor developer experience:
///    - [`clippy`](https://github.com/rust-lang/rust-clippy) would choke on them
///    - compilation times of 5 minutes and more (on fast hardware)
/// - large binary size:
///
///   A simple array of strings, `&[&str]`, adds two [`usize`] in terms of overhead **per
///   [`str`]** (tuple of `(pointer, length)`), which is 16 bytes on 64-bit systems and
///   therefore **longer than the average word** (which sits at around 15 bytes, give or
///   take). Seeing as there can be hundreds of thousands, if not millions of entries,
///   this quickly *doubles* the binary size for no good reason.
/// - not available statically, aka at compile time, aka incurring a runtime cost. This
///   crate's binary is optimized for start-up speed.
///
/// For more info, an overview of the methods tried
/// ([`phf`](https://crates.io/crates/phf) and more), and benchmarks, see [this
/// issue](https://github.com/alexpovel/betterletter-rs/issues/9).
#[derive(Debug, Clone, Copy)]
pub struct GermanStage;

impl Stage for GermanStage {
    fn substitute(&self, input: &str) -> StageResult {
        const INDICATOR: char = '\0';

        debug!("Working on input '{}'", input.escape_debug());

        let mut output = String::with_capacity(input.len());
        let mut machine = StateMachine::new();

        // The state machine, much like a missing trailing newline in a file, will
        // misbehave if the very last transition is not an 'external' one (the last word
        // won't be detected properly).
        for char in input.chars().chain(std::iter::once(INDICATOR)) {
            trace!(
                "Beginning processing of character '{}'",
                char.escape_debug()
            );

            let transition = machine.transition(char);

            trace!("Transition is '{:?}'", transition);

            match transition {
                Transition::External => {
                    output.push(char);
                    continue;
                }
                Transition::Entered | Transition::Internal => {
                    continue;
                }
                Transition::Exited => {
                    debug!("Exited machine: {:?}", machine);

                    let original = machine.current_word().content().to_owned();
                    let word =
                        find_valid_replacement(&original, machine.current_word().replacements())
                            .unwrap_or(original);

                    debug!("Processed word, appending to output: {:?}", &word);
                    output.push_str(&word);

                    // Add back the non-word character that caused the exit transition in the
                    // first place.
                    output.push(char);
                }
            }
        }

        let c = output.pop();
        debug_assert!(
            c == Some(INDICATOR),
            "Trailing indicator byte expected, but found '{c:?}'."
        );

        debug!("Final output string is '{}'", output.escape_debug());

        Ok(output.into())
    }
}

fn find_valid_replacement(word: &str, replacements: &[Replacement]) -> Option<String> {
    let replacement_combinations: Vec<Vec<Replacement>> = replacements
        .iter()
        .powerset()
        .map(|v| v.into_iter().copied().collect())
        .collect();

    debug!("Starting search for valid replacement for word '{}'", word);
    trace!(
        "All replacement combinations to try: {:?}",
        replacement_combinations
    );

    // By definition, the power set contains the empty set. There are two options for
    // handling it:
    // - not skipping: empty set is tried first, and if that word is valid, it is
    //   returned
    // - skipping: empty set is skipped, *some* replacements will take place; if none of
    //   them are valid, no replacements will take place
    //
    // Not skipping it means words like `Busse` will remain unchanged on first
    // iteration. Then, `Busse` will turn out to be valid already and will be returned .
    // Skipping it means `Buße` is tried, which is *also* valid and returned, foregoing
    // `Busse`.
    debug_assert!(replacement_combinations
        .first()
        .map_or(true, std::vec::Vec::is_empty));

    for replacements in replacement_combinations.into_iter().skip(1) {
        let mut candidate = word.to_owned();
        candidate.apply_replacements(replacements);
        trace!(
            "Replaced candidate word, now is: '{}'. Starting validity check.",
            candidate
        );

        if is_valid(&candidate, &contained_in_global_word_list) {
            debug!("Candidate '{}' is valid, returning early", candidate);
            return Some(candidate);
        }

        trace!("Candidate '{}' is invalid, trying next one", candidate);
    }

    debug!("No valid replacement found, returning");
    None
}

fn contained_in_global_word_list(word: &str) -> bool {
    binary_search_uneven(word, VALID_GERMAN_WORDS, '\n')
}

// https://github.com/jaemk/cached/issues/135#issuecomment-1315911572
#[cached(
    type = "SizedCache<String, bool>",
    create = "{ SizedCache::with_size(1024) }",
    convert = r#"{ String::from(word) }"#
)]
fn is_valid(word: &str, predicate: &impl Fn(&str) -> bool) -> bool {
    trace!("Trying candidate '{}'", word);

    let casing = WordCasing::try_from(word);
    trace!("Casing of candidate is '{:?}'", casing);

    match casing {
        Ok(WordCasing::AllLowercase) => {
            // There is no further processing we can/want to do (or is there...
            // https://www.youtube.com/watch?v=HLRdruqQfRk).
            predicate(word)
        }
        Ok(WordCasing::AllUppercase) => {
            // Convert to something sensible before proceeding.
            let tc = word.to_titlecase_lower_rest();
            debug_assert!(
                WordCasing::try_from(tc.as_str()) == Ok(WordCasing::Titlecase),
                "Titlecased word, but isn't categorized correctly."
            );

            is_valid(&tc, predicate)
        }
        Ok(WordCasing::Mixed) => {
            // For MiXeD casing, the word's first character governs its further
            // treatment.
            match word.chars().next() {
                Some(c) if c.is_uppercase() => {
                    let tc = word.to_titlecase_lower_rest();
                    debug_assert!(
                        WordCasing::try_from(tc.as_str()) == Ok(WordCasing::Titlecase),
                        "Titlecased word, but isn't categorized correctly."
                    );

                    is_valid(&tc, predicate)
                }
                _ => is_valid(&word.to_lowercase(), predicate),
            }
        }
        Ok(WordCasing::Titlecase) => {
            // Regular nouns are normally titlecase, so see if they're found
            // immediately (e.g. "Haus").
            predicate(word)
                // Adjectives and verbs might be titlecased at the beginning of
                // sentences etc. (e.g. "Gut gemacht!" -> we need "gut").
                || is_valid(&word.to_lowercase(), predicate)
                // None of these worked: we might have a compound word. These are
                // *never* assumed to occur as anything but titlecase (e.g.
                // "Hausüberfall").
                || is_compound_word(word, predicate)
        }
        Err(_) => false, // Ran into some unexpected characters...
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use common::instrament;
    use itertools::Itertools;
    use rstest::rstest;

    #[test]
    fn test_words_are_sorted() {
        let original = VALID_GERMAN_WORDS.lines().collect_vec();

        let mut sorted = VALID_GERMAN_WORDS.lines().collect_vec();
        sorted.sort_unstable(); // see also: clippy::stable_sort_primitive

        assert_eq!(original, sorted.as_slice());
    }

    #[test]
    fn test_words_are_unique() {
        let original = VALID_GERMAN_WORDS.lines().collect_vec();

        let mut unique = VALID_GERMAN_WORDS.lines().collect_vec();
        unique.sort_unstable(); // see also: clippy::stable_sort_primitive
        unique.dedup();

        assert_eq!(original, unique.as_slice());
    }

    #[test]
    fn test_word_list_is_not_filtered() {
        assert!(
            VALID_GERMAN_WORDS.lines().any(str::is_ascii),
            concat!(
                "Looks like you're using a filtered word list containing only special characters.",
                " The current implementation relies on the full word list (also containing all non-Umlaut words)"
            )
        );
    }

    #[test]
    fn test_is_valid_on_empty_input() {
        assert!(!is_valid("", &contained_in_global_word_list));
    }

    instrament! {
        #[rstest]
        fn test_is_valid(
            #[values(
                "????",
                "\0",
                "\0Dübel",
                "\0Dübel\0",
                "🤩Dübel",
                "🤩Dübel🤐",
                "😎",
                "dröge",
                "DüBeL",
                "Dübel\0",
                "Duebel",
                "kindergarten",
                "Koeffizient",
                "kongruent",
                "Kübel",
                "Mauer",
                "Mauer😂",
                "Mauerdübel",
                "Mauerdübelkübel",
                "Maür",
                "Maürdübelkübel",
                "messgerät",
                "No\nway",
                "Süßwasserschwimmbäder",
                "مرحبا",
                "你好",
            )]
            word: String
        ) (|data: &TestIsValid| {
                insta::assert_yaml_snapshot!(data.to_string(), is_valid(&word, &contained_in_global_word_list));
            }
        )
    }

    instrament! {
        #[rstest]
        fn test_process(
            #[values(
                "\0Kuebel",
                "\0Duebel\0",
                "🤩Duebel",
                "🤩Duebel🤐",
                "Dübel",
                "Abenteuer sind toll!",
                "Koeffizient",
                "kongruent",
                "Ich mag Aepfel, aber nicht Aerger.",
                "Ich mag AEPFEL!! 😍",
                "Wer mag Aepfel?!",
                "Was sind aepfel?",
                "Oel ist ein wichtiger Bestandteil von Oel.",
                "WARUM SCHLIESSEN WIR NICHT AB?",
                "Wir schliessen nicht ab.",
                "WiR sChLieSsEn ab!",
                "WiR sChLiesSEn vieLleEcHt aB.",
                "Suess!",
            )]
            word: String
        ) (|data: &TestProcess| {
                let input = word.clone();
                let result = GermanStage{}.substitute(&input).unwrap();
                insta::assert_yaml_snapshot!(data.to_string(), result.0);
            }
        )
    }
}
