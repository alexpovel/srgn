use log::trace;
use unicode_titlecase::StrTitleCase;

pub fn is_compound_word(word: &str, predicate: &impl Fn(&str) -> bool) -> bool {
    trace!("Checking if word is valid compound word: '{}'", word);

    let indices = word.char_indices().skip(1);

    // Greedily fetch the longest possible prefix. Otherwise, we short-circuit and might
    // end up looking for (for example) "He" of "Heizölrechnung" and its suffix
    // "izölrechnung" (not a word), whereas we could have found "Heizöl" and "Rechnung"
    // instead.
    let mut highest_valid_index = None;
    for (i, _) in indices {
        let prefix = &word[..i];

        if predicate(prefix) {
            highest_valid_index = Some(i);
        }
    }

    match highest_valid_index {
        Some(i) => {
            let suffix = &word[i..];

            trace!(
                "Prefix '{}' found in word list, seeing if suffix '{}' is valid.",
                &word[..i],
                suffix
            );

            let tc = suffix.to_titlecase_lower_rest();
            predicate(&tc)
                || predicate(suffix)
                || is_compound_word(&tc, predicate)
                || is_compound_word(suffix, predicate)
        }
        None => false,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use rstest::rstest;

    const WORDS: &[&str] = &["Süßwasser", "schwimm", "Bäder", "Mauer", "Dübel", "Kübel"];

    #[rstest]
    #[case("Süßwasserschwimmbäder", true)]
    #[case("Mauerdübel", true)]
    #[case("Mauerdübelkübel", true)]
    #[case("Not a compound word", false)]
    #[case("Mauer好", false)]
    #[case("Mauerdjieojoid", false)]
    fn test_is_compound_word(#[case] word: &str, #[case] expected: bool) {
        assert_eq!(is_compound_word(word, &|w| WORDS.contains(&w)), expected);
    }
}
