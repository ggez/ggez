

extern crate skeptic;

fn main() {

    // generates doc tests for guide.
    // skeptic::generate_doc_tests(&[
    //     "docs/guides/HelloGgez.md",
    // ]);

    let mdbook_files = skeptic::markdown_files_of_directory("docs/guides/");
    skeptic::generate_doc_tests(&mdbook_files);
}