use betterletters::stages::GermanStage;
use betterletters::{apply as apply_single, apply_par as apply_rayon, Stage};
use criterion::{criterion_group, criterion_main, BenchmarkId, Criterion, Throughput};

const SAMPLE: &str = include_str!("../tests/samples/german/001.txt");

struct SampleSize {
    n_lines: usize,
    n_sentences_per_line: usize,
    n_bytes_per_line: usize,
}

impl std::fmt::Display for SampleSize {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "{} line(s), {} sentences(s) per line ({} bytes per line)",
            self.n_lines, self.n_sentences_per_line, self.n_bytes_per_line
        )
    }
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

    output.repeat(n_lines)
}

fn criterion_bench(c: &mut Criterion) {
    let mut group = c.benchmark_group("rayon");
    let mut destination = std::io::sink();

    let short_run = false;
    if short_run {
        group.sample_size(10); // Default is 100
        group.warm_up_time(std::time::Duration::from_secs(2)); // Default is 3s
    }

    let stages: Vec<Box<dyn Stage>> = vec![Box::new(GermanStage)];

    for n_lines in [1, 10, 100, 500, 1_000, 5_000, 10_000].into_iter() {
        for n_sentences_per_line in [1, 10, 50, 100, 250, 500, 1_000].into_iter() {
            let text = generate_sample_text(n_lines, n_sentences_per_line);
            let n_bytes_per_line = text.len() / n_lines;

            group.throughput(Throughput::Bytes(text.len() as u64));
            let sample_size = SampleSize {
                n_lines,
                n_sentences_per_line,
                n_bytes_per_line,
            };

            group.bench_with_input(
                BenchmarkId::new("single", &sample_size),
                &text,
                |b, text| {
                    b.iter(|| apply_single(&stages, &mut text.as_bytes(), &mut destination));
                },
            );

            group.bench_with_input(BenchmarkId::new("rayon", &sample_size), &text, |b, text| {
                b.iter(|| apply_rayon(&stages, &mut text.as_bytes(), &mut destination));
            });
        }
    }
    group.finish();
}

criterion_group!(benches, criterion_bench);
criterion_main!(benches);
