use std::fs;
use std::io;
use std::path;

use sdl2;

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
/// TODO: See SDL_GetBasePath and SDL_GetPrefPath!
#[derive(Debug)]
pub struct Filesystem {
    base_path: path::PathBuf,
    user_path: path::PathBuf,
    resource_path: path::PathBuf,
}

/// Represents a file, either in the filesystem, or in the resources zip file,
/// or whatever.
#[derive(Debug)]
pub enum File {
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
        let root_path_string = sdl2::filesystem::base_path().unwrap();
        // BUGGO: We need to have the application ID in this path somehow,
        // except the best place to put it would be in the Conf object...
        // which is loaded using the Filesystem.  Hmmmm.
        // We probably need a Filesystem::config_file_path() function to
        // bootstrap the process.
        let pref_path_string = sdl2::filesystem::pref_path("ggez", "").ok().unwrap();

        let mut root_path = path::PathBuf::from(root_path_string);
        // Ditch the filename (if any)
        if let Some(_) = root_path.file_name() {
            root_path.pop();
        }

        let mut resource_path = root_path.clone();
        resource_path.push("resources");
        // TODO: This should also check for resources.zip
        if !resource_path.exists() || !resource_path.is_dir() {
            let msg_str = format!("'resources' directory not found!  Should be in {:?}", resource_path);
            let message = String::from(msg_str);
            let _ = warn(GameError::ResourceLoadError(message));
        }

        let user_path = path::PathBuf::from(pref_path_string);

        Filesystem {
            resource_path: resource_path,
            base_path: root_path,
            user_path: user_path
        }
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
    fn mongle_path(&self, path: &path::Path) -> Result<path::PathBuf, GameError> {
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

    /// Check whether a path points at a directory.
    pub fn is_dir(&self, path: &path::Path) -> bool {
        match self.mongle_path(path) {
            Ok(p) => p.is_dir(),
            Err(_) => false,
        }
        // TODO: Look in resources.zip, save directory.
    }

    /// Return the full path to the directory containing the exe
    pub fn get_source_dir(&self) -> &path::Path {
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

    /// Returns an iterator over all files and directories in the directory.
    /// Lists the base directory if an empty path is given.
    pub fn read_dir(&self, path: &path::Path) -> io::Result<fs::ReadDir> {
        let dest = self.resource_path.join(path);
        dest.read_dir()
    }
}

mod tests {
    use filesystem::*;
    use std::path;

    fn get_dummy_fs_for_tests() -> Filesystem {
        let mut path = path::PathBuf::from(env!("CARGO_MANIFEST_DIR"));
        path.push("resources");
        let f = Filesystem {
            resource_path: path.clone(),
            user_path: path.clone(),
            base_path: path.clone(),
        };
        f

    }

    #[test]
    fn test_file_exists() {
        let f = get_dummy_fs_for_tests();

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

