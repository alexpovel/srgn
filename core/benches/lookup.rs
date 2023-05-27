use betterletter::util::iteration::binary_search_uneven;
use criterion::{black_box, criterion_group, criterion_main, BenchmarkId, Criterion};
use std::collections::HashSet;

const LONGEST_WORD: &str = "Zufallsschufaeintragseinstellungsverfahrensüberführung";
const WORDS: &[&str] = &[
    "Abdämpfung",
    "Abenteuer",
    "Abschluss",
    "Aerodynamik",
    "Box",
    "Bäder",
    "Dominoeffekt",
    "Dübel",
    "Heizöl",
    "Israel",
    "Jürgen",
    "Kindergarten",
    "Koeffizient",
    "Kämpfer",
    "Kübel",
    "Mauer",
    "Maßstab",
    "Messe",
    "Messgerät",
    "Paß",
    "Poet",
    "Rüben",
    "Rückstoß",
    "Schufaeintrag",
    "Steuerung",
    "Vögel",
    "Wasser",
    "aßen",
    "blöd",
    "dröge",
    "großen",
    "größeren",
    "kongruent",
    "quäkt",
    "quält",
    "schließen",
    "schwimm",
    "süß",
    "zwölf",
    "Äpfel",
    "Ärger",
    "Öl",
    "übel",
    "üben",
    "über",
    LONGEST_WORD,
];

fn generate_hashset(words: Vec<&str>) -> HashSet<&str> {
    HashSet::from_iter(words.iter().cloned())
}

fn generate_padded_string_without_delimiter(words: Vec<&str>, padding: char) -> String {
    let mut out = String::new();

    let max_length = words.iter().map(|word| word.len()).max().unwrap();

    for word in words {
        let padding = String::from(padding).repeat(max_length - word.len());
        out.push_str(&format!("{}{}", word, padding));
    }

    out
}

fn binary_search_padded(word: &str, string: &str, block_size: usize) -> bool {
    let num_blocks = string.len() / block_size;

    let mut left = 0;
    let mut right = num_blocks;

    while left < right {
        let mid = left + (right - left) / 2;

        let start = mid * block_size;
        let end = start + block_size;

        let block = &string[start..end];
        let block_word = block.trim_end();

        match block_word.cmp(word) {
            std::cmp::Ordering::Equal => return true,
            std::cmp::Ordering::Less => left = mid + 1,
            std::cmp::Ordering::Greater => right = mid,
        }
    }

    false
}

pub fn criterion_bench(c: &mut Criterion) {
    let mut group = c.benchmark_group("lookup");
    group.sample_size(10); // Default is 100

    let mut words = WORDS.to_vec();

    words.sort();

    let words_set = generate_hashset(words.clone());

    const DELIMITER: char = '\n';
    let words_single_string_with_delimiter = words.clone().join(&DELIMITER.to_string());

    const PADDING: char = ' ';
    let words_single_padded_string_without_delimiter =
        generate_padded_string_without_delimiter(words.clone(), PADDING);

    for word in &[
        "Abdämpfung",                                      // First word
        "Poet",                                            // Middle word
        "über",                                            // Last word
        "Schufaeintragseinstellungsverfahrensüberführung", // Stupid long word
        "nonexistent",                                     // Nonexistent word
    ] {
        group.throughput(criterion::Throughput::Bytes(word.len() as u64));

        group.bench_with_input(
            BenchmarkId::new("lookup_binary_search_vanilla", word),
            word,
            |b, word| b.iter(|| words.binary_search(black_box(word)).is_ok()),
        );

        group.bench_with_input(BenchmarkId::new("lookup_hashset", word), word, |b, word| {
            b.iter(|| words_set.contains(black_box(word)))
        });

        group.bench_with_input(
            BenchmarkId::new("lookup_binary_search_uneven", word),
            word,
            |b, word| {
                b.iter(|| {
                    binary_search_uneven(
                        black_box(word),
                        &words_single_string_with_delimiter,
                        DELIMITER,
                    )
                })
            },
        );

        group.bench_with_input(
            BenchmarkId::new("lookup_binary_search_padded_string", word),
            word,
            |b, word| {
                b.iter(|| {
                    binary_search_padded(
                        word,
                        &words_single_padded_string_without_delimiter,
                        LONGEST_WORD.len(),
                    )
                })
            },
        );

        match (
            words.binary_search(word).is_ok(),
            words_set.contains(word),
            binary_search_uneven(word, &words_single_string_with_delimiter, DELIMITER),
            binary_search_padded(
                word,
                &words_single_padded_string_without_delimiter,
                LONGEST_WORD.len(),
            ),
        ) {
            (true, true, true, true) => {}
            (false, false, false, false) => {}
            _ => panic!("Mismatch for word '{}'", word),
        }
    }
}

criterion_group!(benches, criterion_bench);
criterion_main!(benches);
