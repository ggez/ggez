//! Provides a portable interface to the filesystem.
//!
//! This module provides access to files in specific places:
//!
//! * The `resources/` subdirectory in the same directory as the
//! program executable, 
//! * The `resources.zip` file in the same
//! directory as the program executable, 
//! * The root folder of the  game's "save" directory which is in a 
//! platform-dependent location,
//! such as `~/.local/share/<author>/<gameid>/` on Linux.  The `gameid`
//! and `author` parts are the strings passed to
//! `Context::load_from_conf()`.
//!
//! Files will be looked for in these locations in order, and the first one
//! found used.  That allows game assets to be easily distributed as an archive
//! file, but locally overridden for testing or modding simply by putting
//! altered copies of them in the game's `resources/` directory.
//!
//! Note that the file lookups WILL follow symlinks!  It is
//! more for convenience than absolute security, so don't treat it as
//! being secure.
//!
//! If you build `ggez` with the `cargo-resource-root` feature flag, it will
//! also look for a `resources/` subdirectory in the same directory as your
//! `Cargo.toml`, which can be very convenient for development.

// BUGGO: TODO: Also make it print out the searched directories when it
// can't find a file!

use std::env;
use std::fmt;
use std::io;
use std::path;

use app_dirs::*;

use GameError;
use GameResult;
use conf;
use vfs::{self, VFS};

const CONFIG_NAME: &'static str = "conf.toml";

/// A structure that contains the filesystem state and cache.
#[derive(Debug)]
pub struct Filesystem {
    vfs: vfs::OverlayFS,
    resources_path: path::PathBuf,
    zip_path: path::PathBuf,
    // user_config_path: path::PathBuf,
    user_data_path: path::PathBuf,
}

/// Represents a file, either in the filesystem, or in the resources zip file,
/// or whatever.
pub enum File {
    VfsFile(Box<vfs::VFile>),
}

impl fmt::Debug for File {
    // TODO: Make this more useful.
    // But we can't seem to get a filename out of a file,
    // soooooo.
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match *self {
            File::VfsFile(ref _file) => write!(f, "VfsFile"),
        }
    }
}

impl io::Read for File {
    fn read(&mut self, buf: &mut [u8]) -> io::Result<usize> {
        match *self {
            File::VfsFile(ref mut f) => f.read(buf),
        }
    }
}



impl io::Write for File {
    fn write(&mut self, buf: &[u8]) -> io::Result<usize> {
        match *self {
            File::VfsFile(ref mut f) => f.write(buf),
        }
    }

    fn flush(&mut self) -> io::Result<()> {
        match *self {
            File::VfsFile(ref mut f) => f.flush(),
        }
    }
}


impl Filesystem {
    /// Create a new Filesystem instance, using the given `id` and (on
    /// some platforms) the `author` as a portion of the user
    /// directory path.  This function is called automatically by
    /// ggez, the end user should never need to call it.
    pub fn new(id: &'static str, author: &'static str) -> GameResult<Filesystem> {
        let app_info = AppInfo {
            name: id,
            author: author,
        };
        let mut root_path = env::current_exe()?;

        // Ditch the filename (if any)
        if root_path.file_name().is_some() {
            root_path.pop();
        }

        // Set up VFS to merge resource path, root path, and zip path.
        let mut overlay = vfs::OverlayFS::new();

        let mut resources_path;
        let mut resources_zip_path;
        let user_data_path;
        // let user_config_path;
        // <game exe root>/resources/
        {
            resources_path = root_path.clone();
            resources_path.push("resources");
            let physfs = vfs::PhysicalFS::new(&resources_path, true);
            overlay.push_back(Box::new(physfs));
        }

        // <root>/resources.zip
        {
            resources_zip_path = root_path.clone();
            resources_zip_path.push("resources.zip");
            if resources_zip_path.exists() {
                let zipfs = vfs::ZipFS::new(&resources_zip_path)?;
                overlay.push_back(Box::new(zipfs));
            }
        }

        // Per-user data dir,
        // ~/.local/share/whatever/
        {
            user_data_path = app_root(AppDataType::UserData, &app_info)?;
            let physfs = vfs::PhysicalFS::new(&user_data_path, true);
            overlay.push_back(Box::new(physfs));
        }

        // // Writeable local dir, ~/.config/whatever/
        // // Save game dir is read-write
        // {
        //     user_config_path = app_root(AppDataType::UserConfig, &app_info)?;
        //     let physfs = vfs::PhysicalFS::new(&user_config_path, false);
        //     overlay.push_back(Box::new(physfs));
        // }

        // Cargo manifest dir!
        #[cfg(feature = "cargo-resource-root")]
        {
            if let Ok(manifest_dir) = env::var("CARGO_MANIFEST_DIR") {
                let mut path = path::PathBuf::from(manifest_dir);
                path.push("resources");
                let physfs = vfs::PhysicalFS::new(&path, false);
                overlay.push_back(Box::new(physfs));
            }
        }

        let fs = Filesystem { 
            vfs: overlay,
            resources_path: resources_path,
            zip_path: resources_zip_path,
            // user_config_path: user_config_path,
            user_data_path: user_data_path,
        };

        Ok(fs)
    }


    /// Opens the given path and returns the resulting `File`
    /// in read-only mode.
    pub fn open<P: AsRef<path::Path>>(&mut self, path: P) -> GameResult<File> {
        self.vfs
            .open(path.as_ref())
            .map(|f| File::VfsFile(f))
    }

    /// Opens a file in the user directory with the given `std::fs::OpenOptions`.
    /// Note that even if you open a file read-only, it can only access
    /// files in the user directory.
    pub fn open_options<P: AsRef<path::Path>>(&mut self,
                                              path: P,
                                              options: &vfs::OpenOptions)
                                              -> GameResult<File> {
        self.vfs
            .open_options(path.as_ref(), options)
            .map(|f| File::VfsFile(f))
            .map_err(|e| {
                GameError::ResourceLoadError(format!("Tried to open {:?} but got error: {:?}",
                                                     path.as_ref(),
                                                     e))
            })
    }

    /// Creates a new file in the user directory and opens it
    /// to be written to, truncating it if it already exists.
    pub fn create<P: AsRef<path::Path>>(&mut self, path: P) -> GameResult<File> {
        self.vfs.create(path.as_ref()).map(|f| File::VfsFile(f))
    }

    /// Create an empty directory in the user dir
    /// with the given name.  Any parents to that directory
    /// that do not exist will be created.
    pub fn create_dir<P: AsRef<path::Path>>(&mut self, path: P) -> GameResult<()> {
        self.vfs.mkdir(path.as_ref())
    }

    /// Deletes the specified file in the user dir.
    pub fn delete<P: AsRef<path::Path>>(&mut self, path: P) -> GameResult<()> {
        self.vfs.rm(path.as_ref())
    }

    /// Deletes the specified directory in the user dir,
    /// and all its contents!
    pub fn delete_dir<P: AsRef<path::Path>>(&mut self, path: P) -> GameResult<()> {
        self.vfs.rmrf(path.as_ref())
    }

    /// Check whether a file or directory exists.
    pub fn exists<P: AsRef<path::Path>>(&self, path: P) -> bool {
        self.vfs.exists(path.as_ref())
    }

    /// Check whether a path points at a file.
    pub fn is_file<P: AsRef<path::Path>>(&self, path: P) -> bool {
        self.vfs
            .metadata(path.as_ref())
            .map(|m| m.is_file())
            .unwrap_or(false)
    }

    /// Check whether a path points at a directory.
    pub fn is_dir<P: AsRef<path::Path>>(&self, path: P) -> bool {
        self.vfs
            .metadata(path.as_ref())
            .map(|m| m.is_dir())
            .unwrap_or(false)
    }

    /// Return the full path to the user data directory
    pub fn get_user_data_dir(&self) -> &path::Path {
        &self.user_data_path
    }

    /// Returns the full path to the resource directory
    /// (even if it doesn't exist)
    pub fn get_resources_dir(&self) -> &path::Path {
        &self.resources_path
    }

    /// Returns an iterator over all files and directories in the resource directory,
    /// in no particular order.
    ///
    /// Lists the base directory if an empty path is given.
    pub fn read_dir<P: AsRef<path::Path>>(&mut self, path: P) -> GameResult<Vec<path::PathBuf>> {
        // TODO: This should return an iterator, and be called iter()
        let itr = self.vfs.read_dir(path.as_ref())?
            .map(|fname| fname.unwrap())
            .collect();
        Ok(itr)
    }

    /// Prints the contents of all data directories.
    /// Useful for debugging.
    pub fn print_all(&mut self) -> GameResult<()> {
        /// TODO: Should tell you which source the resulting files come from...
        for itm in self.read_dir("/")? {
            println!("{:?}", itm);
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
            Err(GameError::ConfigError(String::from("Could not write config file because a directory is in the way?")))
        }
    }
}


#[cfg(test)]
mod tests {
    use error::*;
    use filesystem::*;
    use vfs::*;
    use std::path;
    use std::io::{Read, Write};

    fn get_dummy_fs_for_tests() -> Filesystem {
        let mut path = path::PathBuf::from(env!("CARGO_MANIFEST_DIR"));
        path.push("resources");
        let physfs = vfs::PhysicalFS::new(&path, false);
        let mut ofs = vfs::OverlayFS::new();
        ofs.push_front(Box::new(physfs));
        Filesystem { 
            vfs: ofs,

            resources_path: "".into(),
            zip_path: "".into(),
            // user_config_path: "".into(),
            user_data_path: "".into(),
        }

    }

    #[test]
    fn test_file_exists() {
        let mut f = get_dummy_fs_for_tests();

        let tile_file = path::Path::new("/tile.png");
        assert!(f.exists(tile_file));
        assert!(f.is_file(tile_file));

        let tile_file = path::Path::new("/oglebog.png");
        assert!(!f.exists(tile_file));
        assert!(!f.is_file(tile_file));
        assert!(!f.is_dir(tile_file));

    }

    #[test]
    fn test_read_dir() {
        let mut f = get_dummy_fs_for_tests();

        //let dir_contents_size = f.read_dir().unwrap().len();
        //assert!(dir_contents_size > 0);
    }

    #[test]
    fn test_create_delete_file() {
        let mut fs = get_dummy_fs_for_tests();
        let test_file = path::Path::new("/testfile.txt");
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
            let rel_file = "testfile.txt";
            match fs.open(rel_file) {
                Err(GameError::ResourceNotFound(_, _)) => (),
                Err(e) => panic!("Invalid error for opening file with relative path: {:?}", e),
                Ok(f) => panic!("Should have gotten an error but instead got {:?}!", f),
            }
        }

        {
            // This absolute path should work on Windows too since we
            // completely remove filesystem roots.
            match fs.open("/ooglebooglebarg.txt") {
                Err(GameError::ResourceNotFound(_, _)) => (),
                Err(e) => panic!("Invalid error for opening nonexistent file: {}", e),
                Ok(f) => panic!("Should have gotten an error but instead got {:?}", f),
            }
        }
    }

    //#[test]
    //#fn test_app_dirs() {
    //#use app_dirs::*;
    //#use sdl2;

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
