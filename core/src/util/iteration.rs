use itertools::Itertools;
use std::cmp::Ordering;
use std::str;

pub fn _power_set<C, T>(collection: C) -> Vec<Vec<T>>
where
    C: IntoIterator<Item = T>,
    T: Clone,
{
    power_set_impl(collection, true)
}

pub fn power_set_without_empty<C, T>(collection: C) -> Vec<Vec<T>>
where
    C: IntoIterator<Item = T>,
    T: Clone,
{
    power_set_impl(collection, false)
}

fn power_set_impl<C, T>(collection: C, include_empty_set: bool) -> Vec<Vec<T>>
where
    C: IntoIterator<Item = T>,
    T: Clone,
{
    let vec = collection.into_iter().collect_vec();

    // https://en.wikipedia.org/wiki/Power_set#Properties
    let mut result = Vec::with_capacity(2usize.checked_pow(vec.len() as u32).expect("Overflow"));

    let start = if include_empty_set { 0 } else { 1 };

    for i in start..=vec.len() {
        result.extend(vec.iter().cloned().combinations(i));
    }

    result
}

pub fn binary_search_uneven(needle: &str, haystack: &str, sep: char) -> bool {
    if needle.is_empty() {
        return true;
    }

    if haystack.is_empty() || needle.len() > haystack.len() {
        return false;
    }

    let leftmost = 0;
    let rightmost = haystack.len();

    let mut low = leftmost;
    let mut high = rightmost;

    let haystack = haystack.as_bytes(); // For freely slicing without `.chars()`.

    while low < high {
        let mid = low + (high - low) / 2;

        let pred = |c: &&u8| **c as char == sep;

        let start = match haystack[..mid].iter().rev().find_position(pred) {
            Some((delta, _)) => mid - delta,
            None => leftmost,
        };

        let end = match haystack[mid..].iter().find_position(pred) {
            Some((delta, _)) => mid + delta,
            None => rightmost,
        };

        let haystack_word = str::from_utf8(&haystack[start..end]).unwrap();

        match needle.cmp(haystack_word) {
            Ordering::Less => high = mid.saturating_sub(1),
            Ordering::Equal => return true,
            Ordering::Greater => low = mid + 1,
        }
    }

    false
}

#[cfg(test)]
mod tests {
    use super::{_power_set, power_set_without_empty};
    use common::instrament;
    use rstest::rstest;

    instrament! {
        #[rstest]
        fn test_power_set(
            #[values(vec![], vec![1], vec![1, 2], vec![1, 2, 3])]
            collection: Vec<i32>
        ) (|data: &TestPowerSet| {
            let result = _power_set(collection.clone());
            insta::assert_yaml_snapshot!(data.to_string(), result);
        })
    }

    instrament! {
        #[rstest]
        fn test_power_set_without_empty(
            #[values(vec![], vec![1], vec![1, 2], vec![1, 2, 3])]
            collection: Vec<i32>
        ) (|data: &TestPowerSetWithoutEmpty| {
            let result = power_set_without_empty(collection.clone());
            insta::assert_yaml_snapshot!(data.to_string(), result);
        })
    }

    #[rstest]
    // Base cases, all elements present in any position.
    #[case("abc", "abc,def,ghi,jkl,mno,pqr,stu,vwx,yz", ',', true)]
    #[case("def", "abc,def,ghi,jkl,mno,pqr,stu,vwx,yz", ',', true)]
    #[case("ghi", "abc,def,ghi,jkl,mno,pqr,stu,vwx,yz", ',', true)]
    #[case("jkl", "abc,def,ghi,jkl,mno,pqr,stu,vwx,yz", ',', true)]
    #[case("mno", "abc,def,ghi,jkl,mno,pqr,stu,vwx,yz", ',', true)]
    #[case("pqr", "abc,def,ghi,jkl,mno,pqr,stu,vwx,yz", ',', true)]
    #[case("stu", "abc,def,ghi,jkl,mno,pqr,stu,vwx,yz", ',', true)]
    #[case("vwx", "abc,def,ghi,jkl,mno,pqr,stu,vwx,yz", ',', true)]
    #[case("yz", "abc,def,ghi,jkl,mno,pqr,stu,vwx,yz", ',', true)]
    // Shorter needle than any haystack item.
    #[case("mn", "abc,mno,yz", ',', false)]
    #[case("a", "abc,def,yz", ',', false)]
    #[case("z", "abc,def,yz", ',', false)]
    // Longer needle than any haystack item.
    #[case("abcd", "abc,def,yz", ',', false)]
    #[case("xyz", "abc,def,yz", ',', false)]
    #[case("xyz", "abc,def,yz", ',', false)]
    // Single-character haystack.
    #[case("abc", "a,b,c", ',', false)]
    // Single-character needle and haystack.
    #[case("a", "a,b,c", ',', true)]
    #[case("", "a,b,c", ',', true)]
    #[case("c", "a,b,c", ',', true)]
    #[case("d", "a,b,c", ',', false)]
    // Single-character needle.
    #[case("a", "a,def,yz", ',', true)]
    #[case("a", "abc,def,yz", ',', false)]
    #[case("z", "abc,def,z", ',', true)]
    #[case("z", "abc,def,yz", ',', false)]
    // Repeated-character needle.
    #[case("aaa", "aaa,def,yz", ',', true)]
    #[case("aaa", "abc,def,yz", ',', false)]
    #[case("zzz", "abc,def,zzz", ',', true)]
    #[case("zzz", "abc,def,yz", ',', false)]
    // Empty cases.
    #[case("a", "", ',', false)]
    #[case("", "abc", ',', true)]
    #[case("", "", ',', true)]
    // Switched characters.
    #[case("nmo", "abc,mno,yz", ',', false)]
    #[case("cba", "abc,def,yz", ',', false)]
    // Different separators.
    #[case("abc", "abc-def-yz", '-', true)]
    #[case("abc", "abc\0def\0yz", '\0', true)]
    // Real-world examples.
    #[case("abc", "Hund\nKatze\nMaus", '\n', false)]
    #[case("Hund", "Hund\nKatze\nMaus", '\n', true)]
    #[case("Katze", "Hund\nKatze\nMaus", '\n', true)]
    #[case("Maus", "Hund\nKatze\nMaus", '\n', true)]
    // Real-world examples with multi-byte (UTF-8) characters.
    #[case("Hündin", "Hund\nKatze\nMaus", '\n', false)]
    #[case("Hündin", "Hündin\nKatze\nMaus", '\n', true)]
    #[case("Mäuschen", "Hündin\nKatze\nMäuschen", '\n', true)]
    // Real-world examples with common prefixes.
    #[case("Abdämpfung", "Abdämpfung\nAbenteuer\nAbschluss", '\n', true)]
    #[case("Abdämpfung", "Abdrehen\nAbdämpfung\nAbschluss", '\n', true)]
    fn test_binsearch(
        #[case] needle: &str,
        #[case] haystack: &str,
        #[case] sep: char,
        #[case] expected: bool,
    ) {
        assert_eq!(super::binary_search_uneven(needle, haystack, sep), expected);
    }
}
