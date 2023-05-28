use log::trace;

pub mod instrament;

pub fn titlecase(word: &str) -> String {
    let mut chars = word.chars();
    let mut result = String::with_capacity(word.len());

    if let Some(c) = chars.next() {
        for upper in c.to_uppercase() {
            result.push(upper);
        }
    }

    for c in chars {
        for lower in c.to_lowercase() {
            result.push(lower);
        }
    }

    result
}

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

            predicate(&titlecase(suffix))
                || predicate(suffix)
                || is_compound_word(&titlecase(suffix), predicate)
                || is_compound_word(suffix, predicate)
        }
        None => false,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use rstest::rstest;

    #[rstest]
    #[case("hello", "Hello")]
    #[case("bItTe", "Bitte")]
    #[case("dANKE", "Danke")]
    #[case("übel", "Übel")]
    #[case("uebel", "Uebel")]
    #[case("😀", "😀")]
    #[case("ßuper", "SSuper")]
    #[case("ẞuperduper", "ẞuperduper")]
    #[case("WOW!!", "Wow!!")]
    #[case("ẞß", "ẞß")]
    fn test_titlecase(#[case] word: &str, #[case] expected: &str) {
        assert_eq!(titlecase(word), expected);
    }

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
