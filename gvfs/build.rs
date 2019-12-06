use skeptic;
use std::path::PathBuf;

fn main() {
    let mdbook_files: Vec<PathBuf> = vec!["README.md".into()];
    skeptic::generate_doc_tests(&mdbook_files);
}
