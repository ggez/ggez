use std::env;
use std::fs;
use std::io;
use std::path;

use GameError;
use warn;

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
pub struct Filesystem {
    resource_path: path::PathBuf,
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
    pub fn new() -> Filesystem {
        // BUGGO: We should resolve errors here instead of unwrap.
        let mut root_path = env::current_exe().unwrap();
        // Ditch the filename (if any)
        if let Some(_) = root_path.file_name() {
            root_path.pop();
        }

        // BUGGO: Check for existence of resources path
        root_path.push("resources");

        if !root_path.exists() || !root_path.is_dir() {
            let message = String::from("'resources' directory not found!");
            let _ = warn(GameError::ResourceLoadError(message));
        }

        Filesystem { resource_path: root_path }
    }

    pub fn open(&self, path: &path::Path) -> Result<File, GameError> {

        // Look in resource directory
        let pathbuf = try!(self.mongle_path(path));
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

    /// Takes a relative path and returns an absolute PathBuf
    /// based in the Filesystem's root path.
    /// Sorry, can't think of a better name for this.
    pub fn mongle_path(&self, path: &path::Path) -> Result<path::PathBuf, GameError> {
        if !path.is_relative() {
            let err = GameError::ResourceNotFound(String::from(path.to_str().unwrap()));
            Err(err)
        } else {
            let pathbuf = self.resource_path.join(path);
            Ok(pathbuf)
        }
    }

    /// Check whether a file or directory exists.
    pub fn exists(&self, path: &path::Path) -> bool {
        match self.mongle_path(path) {
            Ok(p) => p.exists(),
            Err(_) => false,
        }
        // TODO: Look in resources.zip, save directory.
    }

    /// Check whether a path points at a file.
    pub fn is_file(&self, path: &path::Path) -> bool {
        match self.mongle_path(path) {
            Ok(p) => p.is_file(),
            Err(_) => false,
        }
        // TODO: Look in resources.zip, save directory.
    }

    /// Check wehther a path points at a directory.
    pub fn is_dir(&self, path: &path::Path) -> bool {
        match self.mongle_path(path) {
            Ok(p) => p.is_dir(),
            Err(_) => false,
        }
        // TODO: Look in resources.zip, save directory.
    }

    /// Return the full path to the directory containing the exe
    pub fn get_source(&self) -> &path::Path {
        &self.resource_path
    }

    /// Return the full path to the user directory
    /// TODO: Make this work
    pub fn get_user_dir(&self) -> &path::Path {
        &self.resource_path
    }

    /// Returns an iterator over all files and directories in the directory.
    /// Lists the base directory if an empty path is given.
    pub fn read_dir(&self, path: &path::Path) -> io::Result<fs::ReadDir> {
        self.get_source().read_dir()
    }
}

#[test]
fn test_filesystem() {
    let f = Filesystem::new();
    let mut root_path = env::current_exe().unwrap();
    root_path.push("resources");

    // I guess it's hard to write tests that rely on external data...
    // assert_eq!(root_path, f.get_user_dir())
}
