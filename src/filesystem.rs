//! A cross-platform interface to the filesystem.
//!
//! This module provides access to files in specific places:
//!
//! * The `resources/` subdirectory in the same directory as the
//! program executable,
//! * The `resources.zip` file in the same
//! directory as the program executable,
//! * The root folder of the  game's "save" directory which is in a
//! platform-dependent location,
//! such as `~/.local/share/<gameid>/` on Linux.  The `gameid`
//! is the the string passed to
//! [`Context::load_from_conf()`](../struct.Context.html#method.load_from_conf); some platforms such as Windows also
//! incorporate the `author` string into the path.
//!
//! Files will be searched for in these locations in order, and the first one
//! found used.  That allows game assets to be easily distributed as an archive
//! file, but locally overridden for testing or modding simply by putting
//! altered copies of them in the game's `resources/` directory.
//!
//! See the source of the `files` example for more details.
//!
//! Note that the file lookups WILL follow symlinks!  It is
//! more for convenience than absolute security, so don't treat it as
//! being secure.
//!
//! If you build `ggez` with the `cargo-resource-root` feature flag, it will
//! also look for a `resources/` subdirectory in the same directory as your
//! `Cargo.toml`, which can be very convenient for development.

use std::env;
use std::fmt;
use std::io;
use std::path;

use app_dirs2::*;

use conf;
use vfs::{self, VFS};
use GameError;
use GameResult;

pub use vfs::OpenOptions;

const CONFIG_NAME: &str = "/conf.toml";

/// A structure that contains the filesystem state and cache.
#[derive(Debug)]
pub struct Filesystem {
    vfs: vfs::OverlayFS,
    resources_path: path::PathBuf,
    zip_path: path::PathBuf,
    user_config_path: path::PathBuf,
    user_data_path: path::PathBuf,
}

/// Represents a file, either in the filesystem, or in the resources zip file,
/// or whatever.
pub enum File {
    /// A wrapper for a VFile trait object.
    VfsFile(Box<vfs::VFile>),
}

impl fmt::Debug for File {
    // Make this more useful?
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
    /// Create a new `Filesystem` instance, using the given `id` and (on
    /// some platforms) the `author` as a portion of the user
    /// directory path.  This function is called automatically by
    /// ggez, the end user should never need to call it.
    pub fn new(id: &'static str, author: &'static str) -> GameResult<Filesystem> {
        let app_info = AppInfo { name: id, author };
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
        let user_config_path;
        // <game exe root>/resources/
        {
            resources_path = root_path.clone();
            resources_path.push("resources");
            trace!("Resources path: {:?}", resources_path);
            let physfs = vfs::PhysicalFS::new(&resources_path, true);
            overlay.push_back(Box::new(physfs));
        }

        // <root>/resources.zip
        {
            resources_zip_path = root_path.clone();
            resources_zip_path.push("resources.zip");
            if resources_zip_path.exists() {
                trace!("Resources zip file: {:?}", resources_zip_path);
                let zipfs = vfs::ZipFS::new(&resources_zip_path)?;
                overlay.push_back(Box::new(zipfs));
            } else {
                trace!("No resources zip file found");
            }
        }

        // Per-user data dir,
        // ~/.local/share/whatever/
        {
            user_data_path = get_app_root(AppDataType::UserData, &app_info)?;
            trace!("User-local data path: {:?}", user_data_path);
            let physfs = vfs::PhysicalFS::new(&user_data_path, true);
            overlay.push_back(Box::new(physfs));
        }

        // Writeable local dir, ~/.config/whatever/
        // Save game dir is read-write
        {
            user_config_path = get_app_root(AppDataType::UserConfig, &app_info)?;
            trace!("User-local configuration path: {:?}", user_config_path);
            let physfs = vfs::PhysicalFS::new(&user_config_path, false);
            overlay.push_back(Box::new(physfs));
        }

        // Cargo manifest dir!
        #[cfg(feature = "cargo-resource-root")]
        {
            if let Ok(manifest_dir) = env::var("CARGO_MANIFEST_DIR") {
                let mut path = path::PathBuf::from(manifest_dir);
                path.push("resources");
                trace!("Cargo manifest resource path: {:?}", user_data_path);
                let physfs = vfs::PhysicalFS::new(&path, true);
                overlay.push_back(Box::new(physfs));
            }
        }

        let fs = Filesystem {
            vfs: overlay,
            resources_path,
            zip_path: resources_zip_path,
            user_config_path,
            user_data_path,
        };

        Ok(fs)
    }

    /// Opens the given path and returns the resulting `File`
    /// in read-only mode.
    pub fn open<P: AsRef<path::Path>>(&mut self, path: P) -> GameResult<File> {
        self.vfs.open(path.as_ref()).map(|f| File::VfsFile(f))
    }

    /// Opens a file in the user directory with the given
    /// [`filesystem::OpenOptions`](struct.OpenOptions.html).
    /// Note that even if you open a file read-only, it can only access
    /// files in the user directory.
    pub fn open_options<P: AsRef<path::Path>>(
        &mut self,
        path: P,
        options: &OpenOptions,
    ) -> GameResult<File> {
        self.vfs
            .open_options(path.as_ref(), options)
            .map(|f| File::VfsFile(f))
            .map_err(|e| {
                GameError::ResourceLoadError(format!(
                    "Tried to open {:?} but got error: {:?}",
                    path.as_ref(),
                    e
                ))
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

    /// Return the full path to the user config directory
    pub fn get_user_config_dir(&self) -> &path::Path {
        &self.user_config_path
    }

    /// Returns the full path to the resource directory
    /// (even if it doesn't exist)
    pub fn get_resources_dir(&self) -> &path::Path {
        &self.resources_path
    }

    /// Returns a list of all files and directories in the resource directory,
    /// in no particular order.
    ///
    /// Lists the base directory if an empty path is given.
    pub fn read_dir<P: AsRef<path::Path>>(
        &mut self,
        path: P,
    ) -> GameResult<Box<Iterator<Item = path::PathBuf>>> {
        let itr = self.vfs.read_dir(path.as_ref())?.map(|fname| {
            fname.expect("Could not read file in read_dir()?  Should never happen, I hope!")
        });
        Ok(Box::new(itr))
    }

    /// Prints the contents of all data directories.
    /// Useful for debugging.
    pub fn print_all(&mut self) {
        for vfs in self.vfs.roots() {
            println!("Source {:?}", vfs);
            match vfs.read_dir(path::Path::new("/")) {
                Ok(files) => for itm in files {
                    println!("  {:?}", itm);
                },
                Err(e) => println!(" Could not read source: {:?}", e),
            }
        }
    }

    /// Outputs the contents of all data directories,
    /// using the "info" log level of the [`log`](https://docs.rs/log/) crate.
    /// Useful for debugging.
    ///
    /// See the [`logging` example](https://github.com/ggez/ggez/blob/master/examples/logging.rs)
    /// for how to collect log information.
    pub fn log_all(&mut self) {
        for vfs in self.vfs.roots() {
            info!("Source {:?}", vfs);
            match vfs.read_dir(path::Path::new("/")) {
                Ok(files) => for itm in files {
                    info!("  {:?}", itm);
                },
                Err(e) => warn!(" Could not read source: {:?}", e),
            }
        }
    }

    /// Adds the given (absolute) path to the list of directories
    /// it will search to look for resources.
    ///
    /// You probably shouldn't use this in the general case, since it is
    /// harder than it looks to make it bulletproof across platforms.
    /// But it can be very nice for debugging and dev purposes, such as
    /// by pushing `$CARGO_MANIFEST_DIR/resources` to it
    pub fn mount(&mut self, path: &path::Path, readonly: bool) {
        let physfs = vfs::PhysicalFS::new(path, readonly);
        trace!("Mounting new path: {:?}", physfs);
        self.vfs.push_back(Box::new(physfs));
    }

    /// Looks for a file named "/conf.toml" in any resource directory and
    /// loads it if it finds it.
    /// If it can't read it for some reason, returns an error.
    pub fn read_config(&mut self) -> GameResult<conf::Conf> {
        let conf_path = path::Path::new(CONFIG_NAME);
        if self.is_file(conf_path) {
            let mut file = self.open(conf_path)?;
            let c = conf::Conf::from_toml_file(&mut file)?;
            Ok(c)
        } else {
            Err(GameError::ConfigError(String::from(
                "Config file not found",
            )))
        }
    }

    /// Takes a `conf::Conf` object and saves it to the user directory,
    /// overwriting any file already there.
    pub fn write_config(&mut self, conf: &conf::Conf) -> GameResult<()> {
        let conf_path = path::Path::new(CONFIG_NAME);
        let mut file = self.create(conf_path)?;
        conf.to_toml_file(&mut file)?;
        if self.is_file(conf_path) {
            Ok(())
        } else {
            Err(GameError::ConfigError(format!(
                "Failed to write config file at {}",
                conf_path.to_string_lossy()
            )))
        }
    }
}

#[cfg(test)]
mod tests {
    use conf;
    use error::*;
    use filesystem::*;
    use std::io::{Read, Write};
    use std::path;

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
            user_config_path: "".into(),
            user_data_path: "".into(),
        }
    }

    #[test]
    fn test_file_exists() {
        let f = get_dummy_fs_for_tests();

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

        let dir_contents_size = f.read_dir("/").unwrap().count();
        assert!(dir_contents_size > 0);
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

    #[test]
    fn test_write_config() {
        let mut f = get_dummy_fs_for_tests();
        let conf = conf::Conf::new();
        // The config file should end up in
        // the resources directory with this
        match f.write_config(&conf) {
            Ok(_) => (),
            Err(e) => panic!("{:?}", e),
        }
        // Remove the config file!
        f.delete(CONFIG_NAME).unwrap();
    }
}
