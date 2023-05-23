use std::{
    collections::VecDeque,
    io::{BufRead, Error, Write},
    sync::{Arc, Mutex},
    thread,
};

use log::{debug, info};

#[cfg(feature = "de")]
use crate::modules::german::German;
#[cfg(feature = "symbols")]
use crate::modules::TextProcessor;

pub mod modules;
pub mod util;

const EXPECTABLE_MAXIMUM_WORD_LENGTH_BYTES: u8 = 64;
const EXPECTABLE_MAXIMUM_MATCHES_PER_WORD: u8 = 8;

/// German is hard-coded for now as passing trait objects to threads didn't work well.
pub fn process_multi_threaded_german(
    // processors: &Vec<Arc<dyn TextProcessor>>,
    source: impl BufRead,
    destination: &mut impl Write,
) -> Result<(), Error> {
    let lines = source.lines().collect::<Result<Vec<_>, _>>()?;

    // let _procs = Arc::new(Mutex::new(processors));

    let num_threads = 8;

    let queue = Arc::new(Mutex::new(
        lines.into_iter().enumerate().collect::<VecDeque<_>>(),
    ));

    let results = Arc::new(Mutex::new(Vec::new()));

    info!("Starting processing.");

    let handles: Vec<_> = (0..num_threads)
        .map(|i| {
            let queue_clone = Arc::clone(&queue);
            let results_clone = Arc::clone(&results);
            // let processors_clone = Arc::clone(&procs);

            thread::spawn(move || {
                while let Some((index, mut item)) = {
                    let mut queue = queue_clone.lock().unwrap();
                    queue.pop_front()
                } {
                    // let processors = processors_clone.lock().unwrap();
                    // for processor in processors.iter() {
                    //     processor.process(&mut item).unwrap();
                    // }

                    let processor = German;
                    processor.process(&mut item).unwrap();

                    let mut results = results_clone.lock().unwrap();
                    info!("Thread {} finished processing line.", i);
                    results.push((index, item));
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

pub fn process_single_threaded(
    processors: &Vec<Box<dyn TextProcessor>>,
    source: &mut impl BufRead,
    destination: &mut impl Write,
) -> Result<(), Error> {
    let mut buf = String::new();

    const EOF_INDICATOR: usize = 0;

    while source.read_line(&mut buf)? > EOF_INDICATOR {
        debug!("Starting processing line: {}", buf.escape_debug());

        for processor in processors {
            processor.process(&mut buf)?;
        }

        debug!("Processed line, will write out: '{}'", buf.escape_debug());
        destination.write_all(buf.as_bytes())?;
        buf.clear();
    }

    info!("Exiting.");
    Ok(())
}
