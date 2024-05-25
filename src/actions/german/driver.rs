use crate::actions::{
    german::{
        machine::{StateMachine, Transition},
        words::{Replace, Replacement, WordCasing},
    },
    Action,
};
use cached::proc_macro::cached;
use cached::SizedCache;
use decompound::{decompound, DecompositionOptions};
use itertools::Itertools;
use itertools::MinMaxResult::{MinMax, NoElements, OneElement};
use log::{debug, trace};
use once_cell::sync::Lazy;
use unicode_titlecase::StrTitleCase;

/// German language action, responsible for Umlauts and Eszett.
///
/// This action is responsible for applying the following rules, [**where
/// applicable**](#example-words-validly-containing-alternative-umlaut-spelling):
/// - [*Umlauts*](https://en.wikipedia.org/wiki/Umlaut_(diacritic)): replace `ue`, `oe`,
///   `ae` with `ü`, `ö`, `ä`, respectively,
/// - [*Eszett*](https://en.wikipedia.org/wiki/%C3%9F): replace `ss` with `ß`.
///
/// Mechanisms are in place to uphold the following properties:
/// - both lower- and uppercase variants are handled correctly,
/// - compound words are handled correctly.
///
/// Towards this, this action does *not* simply replace all occurrences, but performs
/// checks to ensure only valid replacements are made. The core of these checks is an
/// exhaustive word list. The better the word list, the better the results. As such, any
/// errors in processing could be the result of a faulty word list *or* faulty
/// algorithms.
///
/// # Example: A simple greeting, with Umlaut and Eszett
///
/// ```
/// use srgn::actions::{Action, German};
///
/// let action = German::default();
/// let result = action.act("Gruess Gott!");
/// assert_eq!(result, "Grüß Gott!");
/// ```
///
/// # Example: A compound word
///
/// Note that this compound word is *not* part of the word list (that would be an
/// *elaborate* word list!), but is still handled, as its constituents are.
///
/// ```
/// use srgn::actions::{Action, German};
///
/// let action = German::default();
/// let result = action.act("Du Suesswassertagtraeumer!");
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
/// use srgn::actions::{Action, German};
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
///     let action = German::default();
///     let result = action.act(word);
///     assert_eq!(result, word.to_string());
/// }
/// ```
///
/// Note that `ss`/`ß` is not mentioned, as it is handled
/// [elsewhere][`German::new`], dealing with the topic of words with valid
/// alternative *and* special character spellings.
///
/// # Example: Upper- and mixed case
///
/// This action can handle any case, but assumes **nouns are never lower case** (a pretty
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
/// use srgn::actions::{Action, German};
///
/// let action = German::default();
/// let result = action.act("aEpFeL");
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
/// use srgn::actions::{Action, German};
///
/// let action = German::default();
/// let result: String = action.act("AePfEl");
///
/// // OK: MiXeD CaSe words nouns are okay, *if* starting with a capital letter
/// assert_eq!(result, "ÄPfEl");
/// ```
///
/// ## Subexample: other cases
///
/// ```
/// use srgn::actions::{Action, German};
///
/// let action = German::default();
/// let f = |word: &str| -> String {action.act(word)};
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
/// This action handles the German alphabet *only*, and will leave other input bytes
/// untouched. You get to keep your trailing newlines, emojis (also multi-[`char`]
/// ones), and everything else.
///
/// Of course, the input has to be valid UTF-8, as is ensured by its signature
/// ([`str`]).
///
/// ```
/// use srgn::actions::{Action, German};
///
/// let action = German::default();
/// let result = action.act("\0Schoener    你好 Satz... 👋🏻\r\n\n");
/// assert_eq!(result, "\0Schöner    你好 Satz... 👋🏻\r\n\n");
/// ```
///
/// # Performance
///
/// This action is implemented as a [finite state
/// machine](https://en.wikipedia.org/wiki/Finite-state_machine), which means it runs in
/// linear time as well as constant space. It should therefore be quite fast and memory
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
///   A simple array of strings, `&[&str]`, adds two [`usize`] in terms of overhead
///   **per [`str`]** (tuple of `(pointer, length)`), which is 16 bytes on 64-bit
///   systems and therefore **longer than the average word** (which sits at around 15
///   bytes, give or take). Seeing as there can be hundreds of thousands, if not
///   millions of entries, this quickly *doubles* the binary size for no good reason.
/// - not available statically, aka at compile time, aka incurring a runtime cost. This
///   crate's binary is optimized for start-up speed.
///
/// For more info, an overview of the methods tried
/// ([`phf`](https://crates.io/crates/phf) and more), and benchmarks, see [this
/// issue](https://github.com/alexpovel/srgn/issues/9) and [this
/// thread](https://users.rust-lang.org/t/fast-string-lookup-in-a-single-str-containing-millions-of-unevenly-sized-substrings/98040).
#[derive(Debug, Clone, Copy)]
pub struct German {
    prefer_original: bool,
    naive: bool,
}

impl German {
    /// Create a new [`German`].
    ///
    /// # Arguments
    ///
    /// * `prefer_original`: For a tied situation, where an original word and some
    /// replacement are *both* legal, controls which one is returned. See
    /// [below](#example-words-valid-both-in-original-and-replaced-form) for when this
    /// is relevant.
    /// * `naive`: If `true`, perform any possible replacement, regardless of legality
    /// of the resulting word. Useful for names, which are otherwise not modifiable as
    /// they do not occur in dictionaries. See [example](#example-naive-mode).
    ///
    /// ## Example: Words valid both in original and replaced form
    ///
    /// Some words are validly spelled with alternative Umlauts *and* special characters
    /// *in the same position*, such as:
    /// - [Ma**ß**e](https://de.wiktionary.org/wiki/Ma%C3%9Fe): pertaining to
    ///   measurements
    /// - [Ma**ss**e](https://de.wiktionary.org/wiki/Masse): pertaining to mass/weight
    ///
    /// So if a user inputs `Masse` (they can't spell `Maße`, else they wouldn't have
    /// reached for this crate in the first place), what do they mean? Such cases are
    /// tricky, as there isn't an easy solution without reaching for full-blown
    /// [NLP](https://en.wikipedia.org/wiki/Natural_language_processing) or ML, as the
    /// word's context would be required. This action is much too limited for that. A
    /// choice has to be made:
    ///
    /// - do not replace: keep alternative spelling, or
    /// - replace: keep special character spelling.
    ///
    /// This tool chooses the latter, as it seems [the least
    /// astonishing](https://en.wikipedia.org/wiki/Principle_of_least_astonishment) in
    /// the context of this tool, whose entire point is to **make replacements if
    /// they're valid**.
    ///
    /// This is an issue mainly for Eszett (`ß`), as for it, two valid spellings are
    /// much more likely than for Umlauts.
    ///
    /// ```
    /// use srgn::actions::{Action, German};
    ///
    /// for (original, output) in &[
    ///     ("Busse", "Buße"), // busses / penance
    ///     ("Masse", "Maße"), // mass / measurements
    /// ] {
    ///     let mut action = German::default();
    ///     action.prefer_replacement();
    ///     let result = action.act(original);
    ///     assert_eq!(result, output.to_string());
    ///
    ///    let mut action = German::default();
    ///    action.prefer_original();
    ///    let result = action.act(original);
    ///    assert_eq!(result, original.to_string());
    /// }
    /// ```
    ///
    /// ## Example: naive mode
    ///
    /// Naive mode is essentially forcing a maximum number of replacements.
    ///
    /// ```
    /// use srgn::actions::{Action, German};
    ///
    /// for (original, output) in &[
    ///     ("Frau Schroekedaek", "Frau Schrökedäk"), // Names are not in the word list
    ///     ("Abenteuer", "Abenteür"), // Illegal, but possible now
    /// ] {
    ///    let mut action = German::default();
    ///    action.naive();
    ///    let result = action.act(original);
    ///    assert_eq!(result, output.to_string());
    ///
    ///    // However, this is overridden by:
    ///    action.prefer_original();
    ///    let result = action.act(original);
    ///    assert_eq!(result, original.to_string());
    ///
    ///    // The usual behavior:
    ///    let mut action = German::default();
    ///    action.sophisticated();
    ///    let result = action.act(original);
    ///    assert_eq!(result, original.to_string());
    /// }
    /// ```
    ///
    #[must_use]
    pub fn new(prefer_original: bool, naive: bool) -> Self {
        Self {
            prefer_original,
            naive,
        }
    }

    /// Prefer the original word over any replacement.
    pub fn prefer_original(&mut self) -> &mut Self {
        self.prefer_original = true;
        self
    }

    /// Prefer any replacement over the original word.
    pub fn prefer_replacement(&mut self) -> &mut Self {
        self.prefer_original = false;
        self
    }

    /// Be naive.
    pub fn naive(&mut self) -> &mut Self {
        self.naive = true;
        self
    }

    /// Stop being naive.
    pub fn sophisticated(&mut self) -> &mut Self {
        self.naive = false;
        self
    }
}

impl Default for German {
    /// Create a new [`German`] with default settings.
    ///
    /// Default settings are such that features of this action are leveraged most.
    fn default() -> Self {
        let prefer_original = false;
        let naive = false;
        Self::new(prefer_original, naive)
    }
}

impl Action for German {
    fn act(&self, input: &str) -> String {
        const INDICATOR: char = '\0';

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
                    let word = find_valid_replacement(
                        &original,
                        machine.current_word().replacements(),
                        self.prefer_original,
                        self.naive,
                    )
                    .unwrap_or(original);

                    debug!("Processed word, appending to output: {:?}", &word);
                    output.push_str(&word);

                    // Add back the non-word character that caused the exit transition
                    // in the first place.
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

        output
    }
}

fn find_valid_replacement(
    word: &str,
    replacements: &[Replacement],
    prefer_original: bool,
    naive: bool,
) -> Option<String> {
    let replacement_combinations = {
        let mut res: Vec<Vec<_>> = replacements
            .iter()
            .powerset()
            .map(|v| v.into_iter().cloned().collect())
            .collect();

        if naive {
            // Removes all intermediate sets: the shortest (empty) set is required later
            // for `prefer_original`. The longest contains *all* theoretically possible
            // replacements
            res = match res.into_iter().minmax_by_key(Vec::len) {
                NoElements => {
                    unreachable!("powerset always contains at least the empty set")
                }
                OneElement(e) => vec![e],
                MinMax(min, max) => vec![min, max],
            };
        }

        res
    };

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
    debug_assert!(replacement_combinations.first().map_or(true, Vec::is_empty));

    #[allow(clippy::bool_to_int_with_if)] // Readability is much better.
    let n_skip = if prefer_original { 0 } else { 1 };

    for replacements in replacement_combinations.into_iter().skip(n_skip) {
        let mut candidate = word.to_owned();
        candidate.apply_replacements(replacements);
        trace!(
            "Replaced candidate word, now is: '{}'. Starting validity check.",
            candidate
        );

        if naive || is_valid(&candidate, &contained_in_global_word_list) {
            debug!("Candidate '{}' is valid, returning early", candidate);
            return Some(candidate);
        }

        trace!("Candidate '{}' is invalid, trying next one", candidate);
    }

    debug!("No valid replacement found, returning");
    None
}

static SET: Lazy<fst::Set<&[u8]>> = Lazy::new(|| {
    let bytes: &'static [u8] = include_bytes!(concat!(env!("OUT_DIR"), "/de.fst")); // Generated in `build.rs`.
    trace!("Loading FST.");
    let set = fst::Set::new(bytes).expect("Failed to load FST; FST bytes malformed at build time?");
    trace!("Done loading FST.");
    set
});

fn contained_in_global_word_list(word: &str) -> bool {
    trace!("Performing lookup of '{word}' in FST.");
    let result = SET.contains(word);
    trace!("Done performing word lookup in FST (got '{result}').");

    result
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
            // However, due to how the lookup is generated and deduplicated, words
            // like `süßes` *might not be found* when looked up as a whole. It has
            // been split to `süß` and `es`, and *only these* are in the word list.
            // `süßes` is therefore a compound word, by our definition (it's not, it
            // just falls victim to an imperfect algorithm).
            || decompound(word, predicate, DecompositionOptions::TRY_TITLECASE_SUFFIX).is_ok()
        }
        Ok(WordCasing::AllUppercase) => {
            // Convert to something sensible before proceeding.
            let tc = word.to_titlecase_lower_rest();
            debug_assert!(
                // Infinite recursion should this go wrong, so check
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
                        // Infinite recursion should this go wrong, so check
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
                // None of these worked: we might have a compound word. In the ordinary
                // case, these only occur as titlecase, as they're nouns (e.g.
                // "Hausüberfall").
                || decompound(word, predicate, DecompositionOptions::TRY_TITLECASE_SUFFIX).is_ok()
        }
        Err(_) => false, // Ran into some unexpected characters...
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use rstest::rstest;

    #[test]
    fn test_word_list_is_not_filtered() {
        let mut stream = SET.stream();

        assert!(
            {
                let mut has_any_ascii = false;

                while let Some(key) = fst::Streamer::next(&mut stream) {
                    if key.is_ascii() {
                        has_any_ascii = true;
                        break;
                    }
                }
                has_any_ascii
            },
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

    #[rstest]
    // Regular words
    #[case("Koeffizient", true)]
    #[case("kongruent", true)]
    #[case("Mauer", true)]
    #[case("dröge", true)]
    #[case("Kübel", true)]
    //
    // Mixed case is judged by case of initial character
    #[case("DüBeL", true)] // Noun, upper
    #[case("düBeL", false)] // Noun, lower; *not* detected, always illegal
    #[case("dröGE", true)] // Adjective, lower
    #[case("DrÖgE", true)] // Adjective, upper; start of sentence might have this, so legal
    //
    // Junk
    #[case("????", false)]
    #[case("\0", false)]
    #[case("\0Dübel", false)]
    #[case("Dübel\0", false)]
    #[case("\0Dübel\0", false)]
    #[case("🤩Dübel", false)]
    #[case("🤩Dübel🤐", false)]
    #[case("😎", false)]
    #[case("Mauer😂", false)]
    //
    // Alternative Umlaut/Eszett spellings aren't accepted at this point
    #[case("Duebel", false)]
    //
    // Misspellings
    #[case("Maür", false)]
    #[case("Maürdübelkübel", false)]
    //
    // Lowercasing noun isn't valid
    #[case("Messgerät", true)]
    #[case("messgerät", false)]
    //
    // Compound words are supported
    #[case("Mauerdübel", true)]
    #[case("Mauerdübelkübel", true)]
    #[case("Süßwasserschwimmbäder", true)]
    //
    // Foreign languages
    #[case("kindergarten", false)]
    #[case("Kindergarten", true)] // Capitalized in German
    #[case("No\nway", false)]
    #[case("مرحبا", false)]
    #[case("你好", false)]
    fn test_is_valid(#[case] word: &str, #[case] expected: bool) {
        assert_eq!(is_valid(word, &contained_in_global_word_list), expected);
    }

    #[rstest]
    // Regular word
    #[case("Dübel", "Dübel")]
    //
    // Mixed with junk bytes works
    #[case("\0Kuebel", "\0Kübel")]
    #[case("\0Duebel\0", "\0Dübel\0")]
    #[case("🤩Duebel", "🤩Dübel")]
    #[case("🤩Duebel🤐", "🤩Dübel🤐")]
    //
    // Legally alternative Umlaut/Eszett spelled words are not replaced
    #[case("Abenteuer sind toll!", "Abenteuer sind toll!")]
    #[case("Koeffizient", "Koeffizient")]
    #[case("kongruent", "kongruent")]
    //
    // Casing detection works
    #[case(
        "Ich mag Aepfel, aber nicht Aerger.",
        "Ich mag Äpfel, aber nicht Ärger."
    )]
    #[case("Ich mag AEPFEL!! 😍", "Ich mag ÄPFEL!! 😍")]
    #[case("Wer mag Aepfel?!", "Wer mag Äpfel?!")]
    #[case("Was sind aepfel?", "Was sind aepfel?")] // We are not a spellchecker
    //
    // Casing of Eszett works
    #[case("WARUM SCHLIESSEN WIR NICHT AB?", "WARUM SCHLIEẞEN WIR NICHT AB?")]
    #[case("Wir schliessen nicht ab.", "Wir schließen nicht ab.")]
    #[case("WiR sChLieSsEn ab!", "WiR sChLieẞEn ab!")]
    #[case("WiR sChLiesSEn vieLleEcHt aB.", "WiR sChLießEn vieLleEcHt aB.")]
    #[case("Suess!", "Süß!")]
    //
    // Ö works
    #[case(
        "Oel ist ein wichtiger Bestandteil von Oel.",
        "Öl ist ein wichtiger Bestandteil von Öl."
    )]
    fn test_substitution(#[case] input: &str, #[case] expected: &str) {
        let action = German::default();
        let result = action.act(input);
        assert_eq!(result, expected);
    }

    #[rstest]
    // Single letter. Notice the mapping is irreversible.
    #[case("ue", "ü")]
    #[case("uE", "ü")]
    #[case("Ue", "Ü")]
    #[case("UE", "Ü")]
    //
    // Beginning of word
    #[case("uekol", "ükol")]
    #[case("uEkol", "ükol")]
    #[case("Uekol", "Ükol")]
    #[case("UEkol", "Ükol")]
    //
    // Middle of word
    #[case("guessa", "güßa")]
    #[case("gUessa", "gÜßa")]
    #[case("guEssa", "güßa")]
    #[case("gUEssa", "gÜßa")]
    #[case("Guessa", "Güßa")]
    #[case("GUESSA", "GÜẞA")]
    fn test_casing_when_being_naive(#[case] input: &str, #[case] expected: &str) {
        let mut action = German::default();
        action.naive();
        let result = action.act(input);
        assert_eq!(result, expected);
    }
}
