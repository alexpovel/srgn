use betterletter::modules::{german::German, TextProcessor};
use betterletter::{process_multi_threaded_german, process_single_threaded};
use criterion::{criterion_group, criterion_main, BenchmarkId, Criterion, Throughput};
use std::io::{BufRead, Write};

fn process_single_threaded_german(
    mut source: &mut impl BufRead,
    mut destination: &mut impl Write,
) -> Result<(), std::io::Error> {
    let processors: Vec<Box<dyn TextProcessor>> = vec![Box::new(German)];
    process_single_threaded(&processors, &mut source, &mut destination)
}

// https://www.blindtextgenerator.de/
const SAMPLE: &str = include_str!("sample.txt");

struct SampleSize {
    n_lines: usize,
    n_sentences_per_line: usize,
    bytes_per_line: usize,
}

impl SampleSize {
    fn new(n_lines: usize, n_sentences_per_line: usize, bytes_per_line: usize) -> Self {
        Self {
            n_lines,
            n_sentences_per_line,
            bytes_per_line,
        }
    }
}

impl std::fmt::Display for SampleSize {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "{} line(s), {} sentences(s) per line ({} bytes per line)",
            self.n_lines, self.n_sentences_per_line, self.bytes_per_line
        )
    }
}

fn criterion_bench(c: &mut Criterion) {
    let mut group = c.benchmark_group("threading");
    let mut destination = std::io::sink();

    for n_lines in [1, 2, 10, 100].iter() {
        for sentences_per_line in [1, 10, 100, 1_000, 10_000].iter() {
            let text = generate_sample_text(*n_lines, *sentences_per_line);
            let n_bytes_per_line = text.len() / n_lines;

            group.throughput(Throughput::Bytes(text.len() as u64));

            group.bench_with_input(
                BenchmarkId::new(
                    "single",
                    SampleSize::new(*n_lines, *sentences_per_line, n_bytes_per_line),
                ),
                &text,
                |b, text| {
                    b.iter(|| {
                        process_single_threaded_german(&mut text.as_bytes(), &mut destination)
                    });
                },
            );

            group.bench_with_input(
                BenchmarkId::new(
                    "multi",
                    SampleSize::new(*n_lines, *sentences_per_line, n_bytes_per_line),
                ),
                &text,
                |b, text| {
                    b.iter(|| {
                        process_multi_threaded_german(&mut text.as_bytes(), &mut destination)
                    });
                },
            );
        }
    }
    group.finish();
}

fn generate_sample_text(n_lines: usize, sentences_per_line: usize) -> String {
    let mut sample = SAMPLE.lines().cycle();

    let mut output = String::new();
    (0..sentences_per_line)
        .map(|_| sample.next().unwrap())
        .for_each(|line| {
            output.push_str(line);
            output.push(' ');
        });

    eprintln!(
        "Generated sample line (will repeated {} times):\n{}",
        output, n_lines
    );

    output.repeat(n_lines)
}

criterion_group!(benches, criterion_bench);
criterion_main!(benches);
