use log::trace;
use unicode_titlecase::StrTitleCase;

pub fn is_compound_word(
    word: &str,
    predicate: &impl Fn(&str) -> bool,
    constituents: &mut Vec<String>,
) -> bool {
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
            constituents.push(word[..i].to_owned());

            let suffix = &word[i..];

            trace!(
                "Prefix '{}' found in word list, seeing if suffix '{}' is valid.",
                &word[..i],
                suffix
            );

            let tc = suffix.to_titlecase_lower_rest();

            if predicate(&tc) {
                trace!("Suffix '{}' is valid.", tc);
                constituents.push(tc.to_owned());
                true
            } else if predicate(suffix) {
                trace!("Suffix '{}' is valid.", suffix);
                constituents.push(suffix.to_owned());
                true
            } else if is_compound_word(&tc, predicate, constituents) {
                trace!("Suffix '{}' is valid.", tc);
                true
            } else if is_compound_word(suffix, predicate, constituents) {
                trace!("Suffix '{}' is valid.", suffix);
                true
            } else {
                trace!("Suffix '{}' is not valid.", suffix);
                false
            }
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
        assert_eq!(
            is_compound_word(word, &|w| WORDS.contains(&w), &mut vec![]),
            expected
        );
    }
}
