//! Provides an interface to the user's filesystem.
//!
//! This module provides access to files in specific places:
//!
//! * The `resources/` subdirectory in the same directory as the program executable,
//! * The `resources.zip` file in the same directory as the program executable,
//! * The root folder of the game's `save` directory (eventually) which is in a
//! platform-dependent location
//!
//! Files will be looked for in these places that order.
//!
//! Right now files are read-only.  When we can write files, they will be written
//! to the game's save directory.

use std::fmt;
use std::fs;
use std::io;
use std::path;

use sdl2;

use GameError;
use GameResult;
use warn;

use zip;

/// A structure that contains the filesystem state and cache.
#[derive(Debug)]
pub struct Filesystem {
    base_path: path::PathBuf,
    user_path: path::PathBuf,
    resource_path: path::PathBuf,
    resource_zip: Option<zip::ZipArchive<fs::File>>,
}



/// Represents a file, either in the filesystem, or in the resources zip file,
/// or whatever.
pub enum File<'a> {
    FSFile(fs::File),
    ZipFile(zip::read::ZipFile<'a>),
}

impl<'a> fmt::Debug for File<'a> {
    // TODO: Make this more useful.
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match *self {
            File::FSFile(ref _file) => write!(f, "File"),
            File::ZipFile(ref _file) => write!(f, "Zipfile"),
        }
    }
}

impl<'a> io::Read for File<'a> {
    fn read(&mut self, buf: &mut [u8]) -> io::Result<usize> {
        match *self {
            File::FSFile(ref mut f) => f.read(buf),
            File::ZipFile(ref mut f) => f.read(buf),
        }
    }
}

fn convenient_path_to_str(path: &path::Path) -> GameResult<&str> {
    let errmessage = String::from("Invalid path format");
    let error = GameError::FilesystemError(errmessage);
    path.to_str()
        .ok_or(error)
}

impl Filesystem {
    /// Create a new Filesystem instance.
    pub fn new() -> GameResult<Filesystem> {
        let root_path_string = try!(sdl2::filesystem::base_path());
        // BUGGO: We need to have the application ID in this path somehow,
        // except the best place to put it would be in the Conf object...
        // which is loaded using the Filesystem.  Hmmmm.
        // We probably need a Filesystem::config_file_path() function to
        // bootstrap the process.
        let pref_path_string = try!(sdl2::filesystem::pref_path("ggez", ""));

        let mut root_path = path::PathBuf::from(root_path_string);
        // Ditch the filename (if any)
        if let Some(_) = root_path.file_name() {
            root_path.pop();
        }

        let mut resource_path = root_path.clone();
        resource_path.push("resources");
        if !resource_path.exists() || !resource_path.is_dir() {
            let msg_str = format!("'resources' directory not found!  Should be in {:?}",
                                  resource_path);
            let message = String::from(msg_str);
            let _ = warn(GameError::ResourceNotFound(message));
        }

        // Check for resources zip file.
        let mut resource_zip = None;
        let mut resource_zip_path = root_path.clone();
        resource_zip_path.push("resources.zip");
        if !resource_zip_path.exists() || !resource_zip_path.is_file() {
            let msg_str = format!("'resources.zip' file not found!  Should be in {:?}",
                                  resource_zip_path);
            let message = String::from(msg_str);
            let _ = warn(GameError::ResourceNotFound(message));
        } else {
            // We keep this file open so we don't have to re-parse
            // the zip file every time we load something out of it.
            let f = fs::File::open(resource_zip_path).unwrap();
            let z = zip::ZipArchive::new(f).unwrap();
            resource_zip = Some(z);
        }

        let user_path = path::PathBuf::from(pref_path_string);

        let fs = Filesystem {
            resource_path: resource_path,
            base_path: root_path,
            user_path: user_path,
            resource_zip: resource_zip,
        };

        Ok(fs)
    }


    /// Opens the given path and returns the resulting `File`
    pub fn open(&mut self, path: &path::Path) -> GameResult<File> {

        // Look in resource directory
        let pathbuf = try!(self.mongle_path(path));
        if pathbuf.is_file() {
            let f = try!(fs::File::open(pathbuf));
            return Ok(File::FSFile(f));
        }

        // Look in resources.zip
        if let Some(ref mut zipfile) = self.resource_zip {
            let name = path.to_str().unwrap();
            let f = zipfile.by_name(name).unwrap();
            return Ok(File::ZipFile(f));
        }

        // TODO: Look in save directory

        // Welp, can't find it.
        let errmessage = try!(convenient_path_to_str(path));
        Err(GameError::ResourceNotFound(String::from(errmessage)))
    }

    /// Takes a relative path and returns an absolute PathBuf
    /// based in the Filesystem's root path.
    /// TODO: Sorry, can't think of a better name for this.
    fn mongle_path(&self, path: &path::Path) -> GameResult<path::PathBuf> {
        if !path.is_relative() {
            let pathstr = try!(convenient_path_to_str(path));
            let err = GameError::ResourceNotFound(String::from(pathstr));
            Err(err)
        } else {
            let pathbuf = self.resource_path.join(path);
            Ok(pathbuf)
        }
    }

    /// Check whether a file or directory exists.
    pub fn exists(&mut self, path: &path::Path) -> bool {
        if let Ok(p) = self.mongle_path(path) {
            p.exists()
        } else {
            let name = path.to_str().unwrap();
            if let Some(ref mut zipfile) = self.resource_zip {
                zipfile.by_name(name).is_ok()
            } else {
                false
            }
        }
        // TODO: Look in save directory.
    }

    /// Check whether a path points at a file.
    pub fn is_file(&mut self, path: &path::Path) -> bool {
        if let Ok(p) = self.mongle_path(path) {
            p.is_file()
        } else {
            let name = path.to_str().unwrap();
            if let Some(ref mut zipfile) = self.resource_zip {
                zipfile.by_name(name).is_ok()
            } else {
                false
            }
        }
        // TODO: Look in save directory.
    }

    /// Check whether a path points at a directory.
    pub fn is_dir(&mut self, path: &path::Path) -> bool {
        if let Ok(p) = self.mongle_path(path) {
            p.is_dir()
        } else {
            let name = path.to_str().unwrap();
            if let Some(ref mut zipfile) = self.resource_zip {
                // BUGGO: This doesn't actually do what we want...
                // Zip files don't actually store directories,
                // they just fake it.
                // What we COULD do is iterate through all files
                // in the zip file looking for one with the same
                // name prefix as the directory path?
                zipfile.by_name(name).is_ok()
            } else {
                false
            }
        }
        // TODO: Look in save directory.
    }

    /// Return the full path to the directory containing the exe
    pub fn get_root_dir(&self) -> &path::Path {
        &self.base_path
    }

    /// Return the full path to the user directory
    pub fn get_user_dir(&self) -> &path::Path {
        &self.user_path
    }

    /// Returns the full path to the resource directory
    /// (even if it doesn't exist)
    pub fn get_resource_dir(&self) -> &path::Path {
        &self.resource_path
    }

    /// Returns an iterator over all files and directories in the resource directory,
    /// in no particular order.
    ///
    /// Lists the base directory if an empty path is given.
    ///
    /// TODO: Make it iterate over the zip file as well!
    pub fn read_dir(&self, path: &path::Path) -> io::Result<fs::ReadDir> {
        let dest = self.resource_path.join(path);
        dest.read_dir()
    }

    /// TODO: This should return an iterator, and be called iter()
    pub fn print_all(&mut self) {
        let p = self.resource_path.clone();
        if p.is_dir() {
            let paths = fs::read_dir(p).unwrap();
            for path in paths {
                println!("Resources dir, filename {}", path.unwrap().path().display());
            }
        }

        if let Some(ref mut zipfile) = self.resource_zip {
            for i in 0..zipfile.len() {
                let file = zipfile.by_index(i).unwrap();
                println!("Zip, filename: {}", file.name());
            }
        }
    }
}

mod tests {
    use filesystem::*;
    use std::path;

    #[allow(dead_code)]
    fn get_dummy_fs_for_tests() -> Filesystem {
        let mut path = path::PathBuf::from(env!("CARGO_MANIFEST_DIR"));
        path.push("resources");
        Filesystem {
            resource_path: path.clone(),
            user_path: path.clone(),
            base_path: path.clone(),
            resource_zip: None,
        }

    }

    #[test]
    fn test_file_exists() {
        let mut f = get_dummy_fs_for_tests();

        let tile_file = path::Path::new("tile.png");
        assert!(f.exists(tile_file));
        assert!(f.is_file(tile_file));
    }

    #[test]
    fn test_read_dir() {
        let f = get_dummy_fs_for_tests();

        let dir_contents_size = f.read_dir(path::Path::new("")).unwrap().count();
        assert!(dir_contents_size > 0);

    }
}
