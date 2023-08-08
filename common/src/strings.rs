use std::collections::HashSet;

use log::trace;
use unicode_titlecase::StrTitleCase;

pub fn decompose_compound_word<T: AsRef<str>>(
    word: T,
    is_valid_single_word: &impl Fn(&str) -> bool,
) -> Option<Vec<String>> {
    let word = word.as_ref();
    let mut constituents = vec![];

    fn _is_compound_word(
        word: &str,
        is_valid_single_word: &impl Fn(&str) -> bool,
        constituents: &mut Vec<String>,
    ) -> bool {
        trace!("Checking if word is valid compound word: '{}'", word);

        // Greedily fetch the longest possible prefix. Otherwise, we short-circuit and
        // might end up looking for (for example) "He" of "Heizölrechnung" and its
        // suffix "izölrechnung" (not a word), whereas we could have found "Heizöl" and
        // "Rechnung" instead.
        let mut valid_greedy_split = None;

        for (i, _) in word.char_indices().skip(1) {
            let (prefix, suffix) = word.split_at(i);

            if is_valid_single_word(prefix) {
                valid_greedy_split = Some((prefix, suffix));
            }
        }

        match valid_greedy_split {
            Some((prefix, suffix)) => {
                constituents.push(prefix.to_owned());

                trace!(
                    "Prefix '{}' found to be valid, seeing if suffix '{}' is valid.",
                    prefix,
                    suffix
                );

                let suffixes_to_try: HashSet<String> =
                    // Dedupes in case e.g. titlecasing the suffix results in the same
                    // word
                    vec![suffix.to_owned(), suffix.to_titlecase_lower_rest()]
                        .into_iter()
                        .collect();

                for suffix in suffixes_to_try {
                    if is_valid_single_word(&suffix) {
                        trace!("Suffix '{}' is valid: valid single word", suffix);
                        constituents.push(suffix);
                        return true;
                    }

                    if _is_compound_word(&suffix, is_valid_single_word, constituents) {
                        trace!("Suffix '{}' is valid: valid compound word", suffix);
                        // Not pushing to constituents, that's already been done in the
                        // recursion step
                        return true;
                    }
                }

                trace!("Suffix '{}' is not valid", suffix);
                false
            }
            None => false,
        }
    }

    if _is_compound_word(word, is_valid_single_word, &mut constituents) {
        trace!(
            "Word '{}' is a compound word, consisting of: {:?}",
            word,
            constituents
        );
        Some(constituents)
    } else {
        trace!("Word '{}' is not a compound word", word);
        None
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use rstest::rstest;

    const WORDS: &[&str] = &["Süßwasser", "schwimm", "Bäder", "Mauer", "Dübel", "Kübel"];

    #[rstest]
    #[case("Süßwasserschwimmbäder", Some(vec!["Süßwasser", "schwimm", "Bäder"]))]
    #[case("Mauerdübel", Some(vec!["Mauer", "Dübel"]))]
    #[case("Mauerdübelkübel", Some(vec!["Mauer", "Dübel", "Kübel"]))]
    #[case("Not a compound word", None)]
    #[case("Mauer好", None)]
    #[case("Mauerdjieojoid", None)]
    fn test_is_compound_word(#[case] word: &str, #[case] expected: Option<Vec<&str>>) {
        let expected: Option<Vec<String>> =
            expected.map(|v| v.into_iter().map(|s| s.to_owned()).collect());
        assert_eq!(
            decompose_compound_word(word, &|w| WORDS.contains(&w)),
            expected
        );
    }
}
