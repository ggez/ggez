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

use std::env;
use std::fmt;
use std::fs;
use std::io;
use std::path;

use app_dirs::*;

use GameError;
use GameResult;
use conf;
// use warn;

use zip;


const CONFIG_NAME: &'static str = "conf.toml";
const INVALID_FILENAME: &'static str = "This invalid filename will never exist (hopefully) and if \
                                        you manage to create a file that does have this name and \
                                        it causes mysterious trouble, well, congratulations!";



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

unsafe impl<'a> Send for File<'a> {}

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

impl<'a> io::Seek for File<'a> {
    fn seek(&mut self, pos: io::SeekFrom) -> io::Result<u64> {
        match *self {
            File::FSFile(ref mut f) => f.seek(pos),
            // BUGGO: Zip files don't implement Seek?
            File::ZipFile(ref mut _f) => {
                let err = io::Error::new(io::ErrorKind::Other,
                                         "can't seek in zip files apparently");
                Err(err)
            }
        }
    }
}

fn convenient_path_to_str(path: &path::Path) -> GameResult<&str> {
    let errmessage = format!("Invalid path format for resource: {:?}", path);
    let error = GameError::FilesystemError(errmessage);
    path.to_str()
        .ok_or(error)
}

impl Filesystem {
    /// Create a new Filesystem instance, using
    /// the given `id` as a portion of the user directory path.
    /// This function is called automatically by ggez, the end user
    /// should never need to call it.
    pub fn new(_id: &str) -> GameResult<Filesystem> {
        // BUGGO: AppInfo.id needs to be a &'static str which is bogus!
        // See https://github.com/AndyBarron/app-dirs-rs/issues/19
        let app_info = AppInfo {
            name: "placeholder id",
            author: "ggez",
        };
        let mut root_path = env::current_exe()?;
        let pref_path = get_app_root(AppDataType::UserData, &app_info)?;
        // Ditch the filename (if any)
        if let Some(_) = root_path.file_name() {
            root_path.pop();
        }

        let mut resource_path = root_path.clone();
        resource_path.push("resources");
        if !resource_path.exists() || !resource_path.is_dir() {
            // Do we want to warn here?  ...maybe not.
        }

        // Check for resources zip file.
        let mut resource_zip = None;
        let mut resource_zip_path = root_path.clone();
        resource_zip_path.push("resources.zip");
        if !resource_zip_path.exists() || !resource_zip_path.is_file() {
            // Do we want to warn here?  ...maybe not.
        } else {
            // We keep this file open so we don't have to re-parse
            // the zip file every time we load something out of it.
            let f = fs::File::open(resource_zip_path)?;
            let z = zip::ZipArchive::new(f)?;
            resource_zip = Some(z);
        }

        // Get user path, but it doesn't really matter if it
        // doesn't exist for us so there's no real setup.
        let fs = Filesystem {
            resource_path: resource_path,
            base_path: root_path,
            user_path: pref_path,
            resource_zip: resource_zip,
        };

        Ok(fs)
    }


    /// Opens the given path and returns the resulting `File`
    /// in read-only mode.
    pub fn open<P: AsRef<path::Path>>(&mut self, path: P) -> GameResult<File> {

        // Look in resource directory
        let pathref: &path::Path = path.as_ref();
        let pathbuf = self.rel_to_resource_path(pathref)?;
        if pathbuf.is_file() {
            let f = fs::File::open(pathbuf)?;
            return Ok(File::FSFile(f));
        }

        // Look in resources.zip
        if let Some(ref mut zipfile) = self.resource_zip {
            let errmsg = format!("Asked for invalid path inside resources.zip; should never \
                                  happen?");
            let name = pathref.to_str().ok_or(GameError::UnknownError(errmsg))?;
            let f = zipfile.by_name(name)?;
            return Ok(File::ZipFile(f));
            // TODO: add path to zip + path within zip to `tried`
        }

        // Look in user directory
        let pathbuf = self.rel_to_user_path(pathref)?;
        if pathbuf.is_file() {
            let f = fs::File::open(pathbuf)?;
            return Ok(File::FSFile(f));
        }

        // Welp, can't find it.
        let resource_path = self.rel_to_resource_path(pathref)?;
        let user_path = self.rel_to_user_path(pathref)?;
        let mut zip_path = self.zip_path();
        zip_path.push(pathref);
        let tried = vec![resource_path, user_path, zip_path];
        let errmessage = String::from(convenient_path_to_str(pathref)?);
        Err(GameError::ResourceNotFound(errmessage, tried))
    }

    /// Opens a file in the user directory with the given `std::fs::OpenOptions`.
    /// Note that even if you open a file read-only, it can only access
    /// files in the user directory.
    pub fn open_options<P: AsRef<path::Path>>(&mut self,
                                              path: P,
                                              options: fs::OpenOptions)
                                              -> GameResult<File> {
        let pathbuf = self.rel_to_user_path(path.as_ref())?;

        let f = options.open(pathbuf)?;
        Ok(File::FSFile(f))
    }

    /// Creates a new file in the user directory and opens it
    /// to be written to, truncating it if it already exists.
    pub fn create<P: AsRef<path::Path>>(&mut self, path: P) -> GameResult<File> {
        let pathbuf = self.rel_to_user_path(path.as_ref())?;
        let f = fs::File::create(pathbuf)?;
        Ok(File::FSFile(f))
    }

    /// Create an empty directory in the user dir
    /// with the given name.  Any parents to that directory
    /// that do not exist will be created.
    pub fn create_dir<P: AsRef<path::Path>>(&mut self, path: P) -> GameResult<()> {
        let pathbuf = self.rel_to_user_path(path.as_ref())?;
        fs::create_dir_all(pathbuf).map_err(GameError::from)
    }

    /// Deletes the specified file in the user dir.
    pub fn delete<P: AsRef<path::Path>>(&mut self, path: P) -> GameResult<()> {
        let pathbuf = self.rel_to_user_path(path.as_ref())?;
        fs::remove_file(pathbuf).map_err(GameError::from)
    }

    /// Deletes the specified directory in the user dir,
    /// and all its contents!
    pub fn delete_dir<P: AsRef<path::Path>>(&mut self, path: P) -> GameResult<()> {
        let pathbuf = self.rel_to_user_path(path.as_ref())?;
        fs::remove_dir_all(pathbuf).map_err(GameError::from)
    }

    /// Takes a relative path and returns an absolute PathBuf
    /// based in the Filesystem's root path.
    fn rel_to_resource_path<P: AsRef<path::Path>>(&self, path: P) -> GameResult<path::PathBuf> {
        let pathref = path.as_ref();
        if !pathref.is_relative() {
            let pathstr = convenient_path_to_str(pathref)?;
            let errmsg = format!("Could not load resource from path {}, path is not relative.",
                                 pathstr);
            let err = GameError::ResourceLoadError(errmsg);
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
            let pathstr = convenient_path_to_str(pathref)?;
            let errmsg = format!("Could not load resource from path {}, path is not relative.",
                                 pathstr);
            let err = GameError::ResourceLoadError(errmsg);
            Err(err)
        } else {
            let pathbuf = self.user_path.join(pathref);
            Ok(pathbuf)
        }
    }

    /// Constructs a path to the resource zip file.
    fn zip_path(&self) -> path::PathBuf {
        let mut resource_zip_path = self.base_path.clone();
        resource_zip_path.push("resources.zip");
        resource_zip_path
    }

    /// Check whether a file or directory exists.
    pub fn exists<P: AsRef<path::Path>>(&mut self, path: P) -> bool {
        let path = path.as_ref();
        if let Ok(p) = self.rel_to_resource_path(path) {
            p.exists()
        } else if let Ok(p) = self.rel_to_user_path(path) {
            p.exists()
        } else {
            let name = path.to_str().unwrap_or(INVALID_FILENAME);
            // If we have a valid filename,
            // find the thing.
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
            let name = path.to_str().unwrap_or(INVALID_FILENAME);
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
            let name = path.to_str().unwrap_or(INVALID_FILENAME);
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
    pub fn print_all(&mut self) -> GameResult<()> {
        // Print resource files
        {
            let p = self.resource_path.clone();
            if p.is_dir() {
                let paths = fs::read_dir(p)?;
                for path in paths {
                    println!("Resources dir, filename {}", path?.path().display());
                }
            }
        }

        // User dir files
        {
            let p = self.user_path.clone();
            if p.is_dir() {
                let paths = fs::read_dir(p)?;
                for path in paths {
                    println!("User dir, filename {}", path?.path().display());
                }
            }
        }


        if let Some(ref mut zipfile) = self.resource_zip {
            for i in 0..zipfile.len() {
                let file = zipfile.by_index(i)?;
                println!("Zip, filename: {}", file.name());
            }
        }
        Ok(())
    }


    /// Looks for a file named "conf.toml" in the resources directory
    /// loads it if it finds it.
    /// If it can't read it for some reason, returns an error.
    pub fn read_config(&mut self) -> GameResult<conf::Conf> {
        let conf_path = path::Path::new(CONFIG_NAME);
        if self.is_file(conf_path) {
            let mut file = self.open(conf_path)?;
            let c = conf::Conf::from_toml_file(&mut file)?;
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
            let mut file = self.create(conf_path)?;
            conf.to_toml_file(&mut file)

        } else {
            Err(GameError::ConfigError(String::from("Config file not found")))
        }
    }
}

#[cfg(test)]
mod tests {
    use error::*;
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

    #[test]
    fn test_file_not_found() {
        let mut fs = get_dummy_fs_for_tests();
        {
            if let Err(e) = fs.open("/testfile.txt") {
                match e {
                    GameError::ResourceLoadError(_) => (),
                    _ => panic!("Invalid error for opening file with absolute path"),
                }
            } else {
                panic!("Should have gotten an error but didn't!");
            }
        }

        {
            if let Err(e) = fs.open("testfile.txt") {
                match e {
                    GameError::ResourceNotFound(_, _) => (),
                    _ => panic!("Invalid error for opening nonexistent file"),
                }
            } else {
                panic!("Should have gotten an error but didn't!");
            }
        }
    }

    //#[test]
    //#fn test_app_dirs() {
    //#   use app_dirs::*;
    //#   use sdl2;

    //     let app_info = AppInfo{name:"test", author:"ggez"};
    //     println!("user config: {:?}", get_app_root(AppDataType::UserConfig, &app_info));
    //     println!("user cache: {:?}", get_app_root(AppDataType::UserCache, &app_info));
    //     println!("user data: {:?}", get_app_root(AppDataType::UserData, &app_info));

    //     println!("shared config: {:?}", get_app_root(AppDataType::SharedConfig, &app_info));
    //     println!("shared data: {:?}", get_app_root(AppDataType::SharedData, &app_info));

    //     println!("SDL base path: {}", sdl2::filesystem::base_path().unwrap());
    //     println!("SDL pref path: {}", sdl2::filesystem::pref_path("ggez", "id").unwrap());

    // Okay, we want user data for data, user config for config,
    // On Linux these map to:
    // ~/.local/share/test
    // ~/.config/test
    //
    // Plus we should search next to the exe path,
    // AND we should search in env!(CARGO_MANIFEST_DIR) if it exists.
    // (which is a bit hacky since we'll then end up distributing binaries
    // that check in that dir as defined at compile time...  But hmm.)
    //
    // So what we really need is to search in all these places:
    // next-to-executable
    // CARGO_MANIFEST_DIR
    // AppDataType::UserData for read-only data
    // AppDataType::UserConfig for read-write data (saved games, config files)
    // Last, zip file in ANY of the read-only data locations.
    //
    // This is getting complex.
    // We're starting to really need a full VFS layer to properly overlay
    // these things.  Look more at how physfs implements it?
    //
}
