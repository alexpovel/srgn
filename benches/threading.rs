#[cfg(feature = "de")]
use betterletters::stages::GermanStage;
use betterletters::{apply, Stage};
use criterion::{criterion_group, criterion_main, BenchmarkId, Criterion, Throughput};
use log::info;
use std::io::{BufRead, Write};
use std::{
    collections::VecDeque,
    io::Error,
    sync::{Arc, Mutex},
    thread,
};

fn process_single_threaded_german(
    mut source: &mut impl BufRead,
    mut destination: &mut impl Write,
) -> Result<(), std::io::Error> {
    let stages: Vec<Box<dyn Stage>> = vec![Box::new(GermanStage)];
    apply(&stages, &mut source, &mut destination)
}

/// German is hard-coded for now as passing trait objects to threads didn't work well.
pub fn process_multi_threaded_german(
    // stages: &Vec<Arc<dyn Stage>>,
    source: impl BufRead,
    destination: &mut impl Write,
) -> Result<(), Error> {
    let lines = source.lines().collect::<Result<Vec<_>, _>>()?;

    // let _stages = Arc::new(Mutex::new(stages));

    let num_threads = num_cpus::get();

    let queue = Arc::new(Mutex::new(
        lines.into_iter().enumerate().collect::<VecDeque<_>>(),
    ));

    let results = Arc::new(Mutex::new(Vec::new()));

    info!("Starting processing");

    let handles: Vec<_> = (0..num_threads)
        .map(|i| {
            let queue_clone = Arc::clone(&queue);
            let results_clone = Arc::clone(&results);
            // let stages_clone = Arc::clone(&_stages);

            thread::spawn(move || {
                while let Some((index, item)) = {
                    let mut queue = queue_clone.lock().unwrap();
                    queue.pop_front()
                } {
                    // let stages = stages_clone.lock().unwrap();
                    // for stage in stages.iter() {
                    //     stage.process(&mut item).unwrap();
                    // }

                    let stage = GermanStage;
                    let result: String = stage.substitute(&item).unwrap().into();

                    let mut results = results_clone.lock().unwrap();
                    info!("Thread {} finished processing line", i);
                    results.push((index, result));
                }
            })
        })
        .collect();

    for handle in handles {
        handle.join().unwrap();
    }

    let mut results = Arc::try_unwrap(results).unwrap().into_inner().unwrap();
    results.sort_by_key(|&(index, _)| index);

    let results: Vec<String> = results.into_iter().map(|(_, result)| result).collect();

    destination.write_all(results.join("\n").as_bytes())
}

// https://www.blindtextgenerator.de/
const SAMPLE: &str = include_str!("../tests/samples/german/001.txt");

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

fn generate_sample_text(n_lines: usize, sentences_per_line: usize) -> String {
    let mut sample = SAMPLE.lines().cycle();

    let mut output = String::new();
    (0..sentences_per_line)
        .map(|_| sample.next().unwrap())
        .for_each(|line| {
            output.push_str(line);
            output.push(' ');
        });

    // eprintln!(
    //     "Generated sample line (will repeated {} times):\n{}",
    //     output, n_lines
    // );

    output.repeat(n_lines)
}

fn criterion_bench(c: &mut Criterion) {
    let mut group = c.benchmark_group("threading");
    let mut destination = std::io::sink();

    group.sample_size(10);

    for n_lines in [1, 10].iter() {
        for sentences_per_line in [1, 100_000].iter() {
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

criterion_group!(benches, criterion_bench);
criterion_main!(benches);
