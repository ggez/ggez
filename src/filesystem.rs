//! Provides an interface to the user's filesystem.
//!
//! This module provides access to files in specific places:
//!
//! * The `resources/` subdirectory in the same directory as the program executable,
//! * The `resources.zip` file in the same directory as the program executable,
//! * The root folder of the game's "user" directory which is in a
//! platform-dependent location, such as `~/.local/share/ggez/gameid/` on Linux.
//! The `gameid` part is the ID passed to `Game::new()`.
//!
//! Files will be looked for in these locations in order, and the first one
//! found used.  That allows game assets to be easily distributed as an archive
//! file, but locally overridden for testing or modding simply by putting
//! altered copies of them in the game's `resources/` directory.
//!
//! The `resources/` subdirectory and resources.zip files are read-only.
//! Files that are opened for writing using `Filesystem::open_options()`
//! will be created in the `user` directory.

use std::fmt;
use std::fs;
use std::io;
use std::path;

use sdl2;

use GameError;
use GameResult;
use conf;
// use warn;

use zip;


const CONFIG_NAME: &'static str = "conf.toml";


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
    // But we can't seem to get a filename out of a file,
    // soooooo.
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


impl<'a> io::Write for File<'a> {
    fn write(&mut self, buf: &[u8]) -> io::Result<usize> {
        match *self {
            File::FSFile(ref mut f) => f.write(buf),
            File::ZipFile(_) => panic!("Cannot write to a zip file!"),
        }
    }

    fn flush(&mut self) -> io::Result<()> {
        match *self {
            File::FSFile(ref mut f) => f.flush(),
            File::ZipFile(_) => Ok(()),
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
    /// Create a new Filesystem instance, using
    /// the given `id` as a portion of the user directory path.
    /// This function is called automatically by ggez, the end user
    /// should never need to call it.
    pub fn new(id: &str) -> GameResult<Filesystem> {
        let root_path_string = try!(sdl2::filesystem::base_path());
        let pref_path_string = try!(sdl2::filesystem::pref_path("ggez", id));

        let mut root_path = path::PathBuf::from(root_path_string);
        // Ditch the filename (if any)
        if let Some(_) = root_path.file_name() {
            root_path.pop();
        }

        let mut resource_path = root_path.clone();
        resource_path.push("resources");
        if !resource_path.exists() || !resource_path.is_dir() {
            // let msg_str = format!("'resources' directory not found!  Should be in {:?}",
            //                       resource_path);
            // let message = String::from(msg_str);
            // let _ = warn(GameError::ResourceNotFound(message));
        }

        // Check for resources zip file.
        let mut resource_zip = None;
        let mut resource_zip_path = root_path.clone();
        resource_zip_path.push("resources.zip");
        if !resource_zip_path.exists() || !resource_zip_path.is_file() {
            // let msg_str = format!("'resources.zip' file not found!  Should be in {:?}",
            //                       resource_zip_path);
            // let message = String::from(msg_str);
            // let _ = warn(GameError::ResourceNotFound(message));
        } else {
            // We keep this file open so we don't have to re-parse
            // the zip file every time we load something out of it.
            let f = fs::File::open(resource_zip_path).unwrap();
            let z = zip::ZipArchive::new(f).unwrap();
            resource_zip = Some(z);
        }

        // Get user path, but it doesn't really matter if it
        // doesn't exist for us so there's no real setup.
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
    /// in read-only mode.
    pub fn open<P: AsRef<path::Path>>(&mut self, path: P) -> GameResult<File> {

        // Look in resource directory
        let pathref: &path::Path = path.as_ref();
        let pathbuf = try!(self.rel_to_resource_path(pathref));
        if pathbuf.is_file() {
            let f = try!(fs::File::open(pathbuf));
            return Ok(File::FSFile(f));
        }

        // Look in resources.zip
        if let Some(ref mut zipfile) = self.resource_zip {
            let name = pathref.to_str().unwrap();
            let f = zipfile.by_name(name).unwrap();
            return Ok(File::ZipFile(f));
        }

        // Look in user directory
        let pathbuf = try!(self.rel_to_user_path(pathref));
        if pathbuf.is_file() {
            let f = try!(fs::File::open(pathbuf));
            return Ok(File::FSFile(f));
        }

        // Welp, can't find it.
        let errmessage = try!(convenient_path_to_str(pathref));
        Err(GameError::ResourceNotFound(String::from(errmessage)))
    }

    /// Opens a file in the user directory with the given `std::fs::OpenOptions`.
    /// Note that even if you open a file read-only, it can only access
    /// files in the user directory.
    pub fn open_options<P: AsRef<path::Path>>(&mut self,
                                              path: P,
                                              options: fs::OpenOptions)
                                              -> GameResult<File> {
        let pathbuf = try!(self.rel_to_user_path(path.as_ref()));

        let f = try!(options.open(pathbuf));
        Ok(File::FSFile(f))
    }

    /// Creates a new file in the user directory and opens it
    /// to be written to, truncating it if it already exists.
    pub fn create<P: AsRef<path::Path>>(&mut self, path: P) -> GameResult<File> {
        let pathbuf = try!(self.rel_to_user_path(path.as_ref()));
        let f = try!(fs::File::create(pathbuf));
        Ok(File::FSFile(f))
    }

    /// Create an empty directory in the user dir
    /// with the given name.  Any parents to that directory
    /// that do not exist will be created.
    pub fn create_dir<P: AsRef<path::Path>>(&mut self, path: P) -> GameResult<()> {
        let pathbuf = try!(self.rel_to_user_path(path.as_ref()));
        fs::create_dir_all(pathbuf).map_err(GameError::from)
    }

    /// Deletes the specified file in the user dir.
    pub fn delete<P: AsRef<path::Path>>(&mut self, path: P) -> GameResult<()> {
        let pathbuf = try!(self.rel_to_user_path(path.as_ref()));
        fs::remove_file(pathbuf).map_err(GameError::from)
    }

    /// Deletes the specified directory in the user dir,
    /// and all its contents!
    pub fn delete_dir<P: AsRef<path::Path>>(&mut self, path: P) -> GameResult<()> {
        let pathbuf = try!(self.rel_to_user_path(path.as_ref()));
        fs::remove_dir_all(pathbuf).map_err(GameError::from)
    }

    /// Takes a relative path and returns an absolute PathBuf
    /// based in the Filesystem's root path.
    fn rel_to_resource_path<P: AsRef<path::Path>>(&self, path: P) -> GameResult<path::PathBuf> {
        let pathref = path.as_ref();
        if !pathref.is_relative() {
            let pathstr = try!(convenient_path_to_str(pathref));
            let err = GameError::ResourceNotFound(String::from(pathstr));
            Err(err)
        } else {
            let pathbuf = self.resource_path.join(pathref);
            Ok(pathbuf)
        }
    }

    /// Takes a relative path and returns an absolute PathBuf
    /// based in the Filesystem's user directory.
    fn rel_to_user_path<P: AsRef<path::Path>>(&self, path: P) -> GameResult<path::PathBuf> {
        let pathref = path.as_ref();
        if !pathref.is_relative() {
            let pathstr = try!(convenient_path_to_str(pathref));
            let err = GameError::ResourceNotFound(String::from(pathstr));
            Err(err)
        } else {
            let pathbuf = self.user_path.join(pathref);
            Ok(pathbuf)
        }
    }

    /// Check whether a file or directory exists.
    pub fn exists<P: AsRef<path::Path>>(&mut self, path: P) -> bool {
        let path = path.as_ref();
        if let Ok(p) = self.rel_to_resource_path(path) {
            p.exists()
        } else if let Ok(p) = self.rel_to_user_path(path) {
            p.exists()
        } else {
            let name = path.to_str().unwrap();
            if let Some(ref mut zipfile) = self.resource_zip {
                zipfile.by_name(name).is_ok()
            } else {
                false
            }
        }
    }

    /// Check whether a path points at a file.
    pub fn is_file<P: AsRef<path::Path>>(&mut self, path: P) -> bool {
        let path = path.as_ref();
        if let Ok(p) = self.rel_to_resource_path(path) {
            p.is_file()
        } else if let Ok(p) = self.rel_to_user_path(path) {
            p.is_file()
        } else {
            let name = path.to_str().unwrap();
            if let Some(ref mut zipfile) = self.resource_zip {
                zipfile.by_name(name).is_ok()
            } else {
                false
            }
        }
    }

    /// Check whether a path points at a directory.
    pub fn is_dir<P: AsRef<path::Path>>(&mut self, path: P) -> bool {
        let path = path.as_ref();
        if let Ok(p) = self.rel_to_resource_path(path) {
            p.is_dir()
        } else if let Ok(p) = self.rel_to_user_path(path) {
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
    /// And the user dir.  This probably won't happen until
    /// returning `impl Trait` hits stable, honestly.
    pub fn read_dir<P: AsRef<path::Path>>(&self, path: P) -> io::Result<fs::ReadDir> {
        let resource_dest = self.resource_path.join(path.as_ref());
        // let user_dest = self.user_path.join(path);
        resource_dest.read_dir()
        // .map(|iter| iter.chain(user_dest.read_dir()))
    }

    /// Prints the contents of all data directories.
    /// Useful for debugging.
    /// TODO: This should return an iterator, and be called iter()
    pub fn print_all(&mut self) {
        // Print resource files
        {
            let p = self.resource_path.clone();
            if p.is_dir() {
                let paths = fs::read_dir(p).unwrap();
                for path in paths {
                    println!("Resources dir, filename {}", path.unwrap().path().display());
                }
            }
        }

        // User dir files
        {
            let p = self.user_path.clone();
            if p.is_dir() {
                let paths = fs::read_dir(p).unwrap();
                for path in paths {
                    println!("User dir, filename {}", path.unwrap().path().display());
                }
            }
        }


        if let Some(ref mut zipfile) = self.resource_zip {
            for i in 0..zipfile.len() {
                let file = zipfile.by_index(i).unwrap();
                println!("Zip, filename: {}", file.name());
            }
        }
    }


    /// Looks for a file named "conf.toml" in the resources directory
    /// loads it if it finds it.
    /// If it can't read it for some reason, returns an error.
    pub fn read_config(&mut self) -> GameResult<conf::Conf> {
        let conf_path = path::Path::new(CONFIG_NAME);
        if self.is_file(conf_path) {
            let mut file = try!(self.open(conf_path));
            let c = try!(conf::Conf::from_toml_file(&mut file));
            Ok(c)

        } else {
            Err(GameError::ConfigError(String::from("Config file not found")))
        }
    }

    /// Takes a `conf::Conf` object and saves it to the user directory,
    /// overwriting any file already there.
    pub fn write_config(&mut self, conf: &conf::Conf) -> GameResult<()> {
        let conf_path = path::Path::new(CONFIG_NAME);
        if self.is_file(conf_path) {
            let mut file = try!(self.create(conf_path));
            conf.to_toml_file(&mut file)

        } else {
            Err(GameError::ConfigError(String::from("Config file not found")))
        }
    }
}

#[cfg(test)]
mod tests {
    use filesystem::*;
    use std::path;
    use std::io::{Read, Write};

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

    #[test]
    fn test_create_delete_file() {
        let mut fs = get_dummy_fs_for_tests();
        let test_file = path::Path::new("testfile.txt");
        let bytes = "test".as_bytes();

        {
            let mut file = fs.create(test_file).unwrap();
            file.write(bytes).unwrap();
        }
        {
            let mut buffer = Vec::new();
            let mut file = fs.open(test_file).unwrap();
            file.read_to_end(&mut buffer).unwrap();
            assert_eq!(bytes, buffer.as_slice());
        }

        fs.delete(test_file).unwrap();
    }
}
