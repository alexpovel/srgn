use std::{fs, path::Path};

use cargo_metadata::MetadataCommand;

fn main() {
    generate_word_lists();
}

fn generate_word_lists() {
    let metadata = MetadataCommand::new()
        .no_deps()
        .exec()
        .expect("Failed to get cargo metadata.");

    for entry in fs::read_dir(Path::new("data/word-lists")).expect("Need word lists directory.") {
        let source = entry.expect("Failed to access directory item.");

        let target = metadata.target_directory.join("word-lists").join(
            source
                .file_name()
                .to_str()
                .expect("File name is malformed UTF-8."),
        );

        target.parent().map(fs::create_dir_all);

        if !source
            .metadata()
            .expect("Failed to access directory item's metadata.")
            .is_file()
            || target.exists()
        {
            continue;
        }

        let contents = fs::read_to_string(source.path()).expect("Failed to read file.");

        let mut words: Vec<&str> = contents
            .lines()
            .map(|word| word.trim())
            .filter(|&word| has_special_characters(word))
            .collect();

        words.sort();

        fs::write(target, words.join("\n")).expect("Failed to write file.");
    }
}

fn has_special_characters(word: &str) -> bool {
    !word.is_ascii()
}
