use crate::tests;
use crate::*;

use std::io::Write;

#[test]
fn filesystem_create_correct_paths() {
    let (c, _e) = &mut tests::make_context();

    {
        let mut f = filesystem::create(c, "/filesystem_create_path").unwrap();
        let _ = f.write(b"foo").unwrap();
    }
    let userdata_path = filesystem::user_config_dir(c);
    let userdata_path = &mut userdata_path.to_owned();
    userdata_path.push("filesystem_create_path");
    println!("Userdata path: {:?}", userdata_path);
    assert!(userdata_path.is_file());
}
