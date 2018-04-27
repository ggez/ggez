//! This examples shows the basics of how ggez's file handling
//! works.
//!
//! It doesn't use an event loop, it just runs once and exits,
//! printing a bunch of stuff to the console.

extern crate ggez;

use ggez::ContextBuilder;
use ggez::filesystem;
use ggez::conf;
use std::env;
use std::path;
use std::io::{Read, Write};
use std::str;

pub fn main() {
    let mut cb = ContextBuilder::new("ggez_files_example", "ggez");

    // We add the CARGO_MANIFEST_DIR/resources to the filesystems paths so
    // we we look in the cargo project for files.
    // Using a ContextBuilder is nice for this because it means that
    // it will look for a conf.toml or icon file or such in
    // this directory when the Context is created.
    if let Ok(manifest_dir) = env::var("CARGO_MANIFEST_DIR") {
        let mut path = path::PathBuf::from(manifest_dir);
        path.push("resources");
        println!("Adding path {:?}", path);
        cb = cb.add_resource_path(path);
    }

    let ctx = &mut cb.build().unwrap();

    println!("Full filesystem info: {:#?}", ctx.filesystem);

    println!("Resource stats:");
    ctx.print_resource_stats();

    let dir_contents: Vec<_> = ctx.filesystem.read_dir("/").unwrap().collect();
    println!("Directory has {} things in it:", dir_contents.len());
    for itm in dir_contents {
        println!("   {:?}", itm);
    }

    println!();
    println!("Let's write to a file, it should end up in the user config dir");

    let test_file = path::Path::new("/testfile.txt");
    let bytes = b"test";
    {
        let mut file = ctx.filesystem.create(test_file).unwrap();
        file.write_all(bytes).unwrap();
    }
    println!("Wrote to test file");
    {
        let mut options = filesystem::OpenOptions::new();
        options.append(true);
        let mut file = ctx.filesystem.open_options(test_file, &options).unwrap();
        file.write_all(bytes).unwrap();
    }
    println!("Appended to test file");
    {
        let mut buffer = Vec::new();
        let mut file = ctx.filesystem.open(test_file).unwrap();
        file.read_to_end(&mut buffer).unwrap();
        println!(
            "Read from test file: {:?}",
            str::from_utf8(&buffer).unwrap()
        );
    }

    println!();
    println!("Let's read the default conf file");
    if let Ok(_conf) = ctx.filesystem.read_config() {
        println!("Found existing conf file, its contents are:");
        let mut file = ctx.filesystem.open("/conf.toml").unwrap();
        let mut buffer = Vec::new();
        file.read_to_end(&mut buffer).unwrap();
        println!("{}", str::from_utf8(&buffer).unwrap());

        println!("Now deleting it, re-run the example to recreate it.");
        ctx.filesystem.delete("/conf.toml").unwrap();
    } else {
        println!("No existing conf file found, saving one out");
        let c = conf::Conf::new();
        ctx.filesystem.write_config(&c).unwrap();
        println!("Should now be a 'conf.toml' file under user config dir");
    }

    println!();
    println!("Now let's try to read a file that does not exist");
    {
        if let Err(e) = ctx.filesystem.open("/jfkdlasfjdsa") {
            // The error message contains a big hairy list of each
            // directory tried and what error it got from it.
            println!("Got the error: {:?}", e);
        } else {
            println!("Wait, it does exist?  Weird.")
        }
    }
}
