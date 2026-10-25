use pulldown_cmark::{CodeBlockKind, Event, Parser, Tag};
use std::{collections::HashMap, env, fs, path::Path};

// Expand guide snippets into complete examples for Cargo's native doctest runner.
fn rust_blocks(markdown: &str) -> Vec<(String, String)> {
    let mut blocks = Vec::new();
    let mut current = None;
    for event in Parser::new(markdown) {
        match event {
            Event::Start(Tag::CodeBlock(CodeBlockKind::Fenced(info)))
                if info.split(',').next() == Some("rust") =>
            {
                current = Some((info.to_string(), String::new()));
            }
            Event::Text(text) => {
                if let Some((_, code)) = &mut current {
                    code.push_str(&text);
                }
            }
            Event::End(Tag::CodeBlock(_)) => {
                if let Some(block) = current.take() {
                    blocks.push(block);
                }
            }
            _ => (),
        }
    }
    blocks
}

fn main() {
    println!("cargo:rerun-if-changed=../docs/guides");
    let mut paths: Vec<_> = fs::read_dir("../docs/guides")
        .unwrap()
        .map(|entry| entry.unwrap().path())
        .filter(|path| {
            path.extension().is_some_and(|ext| ext == "md")
                && !path.to_string_lossy().ends_with(".templates.md")
        })
        .collect();
    paths.push("../README.md".into());
    paths.sort();

    let mut generated = String::new();
    for path in &paths {
        println!("cargo:rerun-if-changed={}", path.display());
        let template_path = format!("{}.templates.md", path.display());
        println!("cargo:rerun-if-changed={template_path}");
        let templates: HashMap<_, _> = if Path::new(&template_path).exists() {
            rust_blocks(&fs::read_to_string(&template_path).unwrap())
                .into_iter()
                .map(|(info, code)| {
                    let name = info
                        .split(',')
                        .find(|flag| flag.starts_with("template-"))
                        .unwrap();
                    (name.to_owned(), code)
                })
                .collect()
        } else {
            HashMap::new()
        };
        let mut docs = format!("Examples from {}\n\n", path.display());
        for (info, code) in rust_blocks(&fs::read_to_string(path).unwrap()) {
            let mut flags = Vec::new();
            let mut expanded = code.clone();
            for flag in info.split(',') {
                if flag.starts_with("template-") {
                    // Companion .templates.md files wrap snippets at {{example}}.
                    expanded = templates
                        .get(flag)
                        .expect("missing template")
                        .replace("{{example}}", &code);
                } else {
                    flags.push(flag);
                }
            }
            docs.push_str(&format!("```{}\n{expanded}\n```\n\n", flags.join(",")));
        }
        let module = path
            .file_stem()
            .unwrap()
            .to_str()
            .unwrap()
            .to_ascii_lowercase();
        generated.push_str(&format!("#[doc = {docs:?}]\npub mod {module} {{}}\n"));
    }
    fs::write(
        Path::new(&env::var_os("OUT_DIR").unwrap()).join("doc-tests.rs"),
        generated,
    )
    .unwrap();
}
