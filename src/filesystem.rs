use std::env;
use std::fs;
use std::io;
use std::path;

use ::GameError;

/// Provides an interface to the user's filesystem.
///
/// This module provides access to files in specific places:
/// * The `resources/` subdirectory in the same directory as the program executable,
/// * The `resources.zip` file in the same directory as the program executable (eventually),
/// * The root folder of the game's `save` directory (eventually)
///
/// Files will be looked for in these places in order.
///
/// Right now files are read-only.  When we can write files, they will be written
/// to the game's save directory.
///
#[derive(Debug)]
struct Filesystem {
    resource_path: path::PathBuf,
}

impl Filesystem {
    fn new() -> Filesystem {
        // BUGGO: We should resolve errors here instead of unwrap.
        let mut root_path = env::current_exe().unwrap();
        // Ditch the filename (if any)
        if let Some(_) = root_path.file_name() {
            root_path.pop();
        }

        // BUGGO: Check for existence of resources path
        root_path.push("resources");

        Filesystem { resource_path: root_path }
    }
}

/// Represents a file on the filesystem.
#[derive(Debug)]
enum File {
    FSFile(fs::File),
}

impl io::Read for File {
    fn read(&mut self, buf: &mut [u8]) -> io::Result<usize> {
        match *self {
            File::FSFile(ref mut f) => f.read(buf),
        }
    }
}

impl Filesystem {
    fn open(&self, path: &path::Path) -> Result<File, GameError> {
        // All paths are relative.
        if !path.is_relative() {
            let err = GameError::ResourceNotFound(String::from(path.to_str().unwrap()));
            return Err(err);
        }

        // Look in resource directory
        let pathbuf = self.resource_path.join(path);
        if pathbuf.is_file() {
            // BUGGO: Unwrap
            let f = fs::File::open(pathbuf).unwrap();
            return Ok(File::FSFile(f));
        }

        // TODO: Look in resources.zip

        // TODO: Look in save directory

        // Welp, can't find it.
        let errmessage = String::from(path.to_str().unwrap());
        Err(GameError::ResourceNotFound(errmessage))
    }

    /// Check whether a file or directory exists.
    fn exists(&self, path: &path::Path) -> bool {
        false
    }

    /// Check whether a path points at a file.
    fn is_file(&self, path: &path::Path) -> bool {
        false
    }

    /// Check wehther a path points at a directory.
    fn is_dir(&self, path: &path::Path) -> bool {
        false
    }

    /// Return the full path to the directory containing the exe
    fn get_source(&self) {}

    /// Return the full path to the user directory
    fn get_user_dir(&self) {}

    /// Returns an iterator over all files and directories in the directory.
    /// Lists the base directory if an empty path is given.
    fn read_dir(&self, path: &path::Path) {}
}
