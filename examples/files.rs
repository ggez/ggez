//! This examples shows the basics of how ggez's file handling
//! works.

extern crate ggez;
use ggez::*;
use std::path;
use std::io::{Read, Write};
use std::str;

pub fn main() {
    let c = conf::Conf::new();
    let ctx = &mut Context::load_from_conf("ggez_files_example", "ggez", c).unwrap();

    println!("Filesystem paths are:");
    println!("   User data: {:?}", ctx.filesystem.get_user_data_dir());
    println!("   User config: {:?}", ctx.filesystem.get_user_config_dir());

    let dir_contents = ctx.filesystem.read_dir("/").unwrap();
    println!("Directory has {} things in it:", dir_contents.len());
    for itm in dir_contents {
        println!("   {:?}", itm);
    }


    let test_file = path::Path::new("/testfile.txt");
    let bytes = "test".as_bytes();
    {
        let mut file = ctx.filesystem.create(test_file).unwrap();
        file.write(bytes).unwrap();
    }
    println!("Wrote to test file");
    {
        let mut buffer = Vec::new();
        let mut file = ctx.filesystem.open(test_file).unwrap();
        file.read_to_end(&mut buffer).unwrap();
        println!("Read from test file: {:?}", str::from_utf8(&buffer).unwrap());
    }


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
}
