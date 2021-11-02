//! A cross-platform interface to the filesystem.
//!
//! This module provides access to files in specific places:
//!
//! * The `resources/` subdirectory in the same directory as the
//! program executable, if any,
//! * The `resources.zip` file in the same
//! directory as the program executable, if any,
//! * The root folder of the  game's "save" directory which is in a
//! platform-dependent location,
//! such as `~/.local/share/<gameid>/` on Linux.  The `gameid`
//! is the the string passed to
//! [`ContextBuilder::new()`](../struct.ContextBuilder.html#method.new).
//! Some platforms such as Windows also incorporate the `author` string into
//! the path.
//!
//! These locations will be searched for files in the order listed, and the first file
//! found used.  That allows game assets to be easily distributed as an archive
//! file, but locally overridden for testing or modding simply by putting
//! altered copies of them in the game's `resources/` directory.  It
//! is loosely based off of the `PhysicsFS` library.
//!
//! See the source of the [`files` example](https://github.com/ggez/ggez/blob/master/examples/files.rs) for more details.
//!
//! The names of `resources/` and `resources.zip` can be changed with the methods
//! [`resources_dir_name`](../struct.ContextBuilder.html#method.resources_dir_name)
//! and
//! [`resources_zip_name`](../struct.ContextBuilder.html#method.resources_zip_name)
//!  of ContextBuilder.
//!
//! Note that the file lookups WILL follow symlinks!  This module's
//! directory isolation is intended for convenience, not security, so
//! don't assume it will be secure.

use std::env;
use std::io;
use std::io::SeekFrom;
use std::path;

use directories::ProjectDirs;

use crate::conf;
use crate::vfs::{self, VFS};
use crate::{Context, GameError, GameResult};

pub use crate::vfs::OpenOptions;

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
#[derive(Debug)]
pub enum File {
    /// A wrapper for a VFile trait object.
    VfsFile(Box<dyn vfs::VFile>),
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

impl io::Seek for File {
    fn seek(&mut self, pos: SeekFrom) -> io::Result<u64> {
        match *self {
            File::VfsFile(ref mut f) => f.seek(pos),
        }
    }
}

impl Filesystem {
    /// Create a new `Filesystem` instance, using the given `id` and (on
    /// some platforms) the `author` as a portion of the user
    /// directory path.  This function is called automatically by
    /// ggez, the end user should never need to call it.
    pub fn new(
        id: &str,
        author: &str,
        resources_dir_name: &str,
        resources_zip_name: &str,
    ) -> GameResult<Filesystem> {
        let mut root_path = env::current_exe()?;

        // Ditch the filename (if any)
        if root_path.file_name().is_some() {
            let _ = root_path.pop();
        }

        // Set up VFS to merge resource path, root path, and zip path.
        let mut overlay = vfs::OverlayFS::new();

        let mut resources_path;
        let mut resources_zip_path;
        let user_data_path;
        let user_config_path;

        let project_dirs = match ProjectDirs::from("", author, id) {
            Some(dirs) => dirs,
            None => {
                return Err(GameError::FilesystemError(String::from(
                    "No valid home directory path could be retrieved.",
                )));
            }
        };

        // <game exe root>/resources/
        {
            resources_path = root_path.clone();
            resources_path.push(resources_dir_name);
            trace!("Resources path: {:?}", resources_path);
            let physfs = vfs::PhysicalFS::new(&resources_path, true);
            overlay.push_back(Box::new(physfs));
        }

        // <root>/resources.zip
        {
            resources_zip_path = root_path;
            resources_zip_path.push(resources_zip_name);
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
            user_data_path = project_dirs.data_local_dir();
            trace!("User-local data path: {:?}", user_data_path);
            let physfs = vfs::PhysicalFS::new(user_data_path, true);
            overlay.push_back(Box::new(physfs));
        }

        // Writeable local dir, ~/.config/whatever/
        // Save game dir is read-write
        {
            user_config_path = project_dirs.config_dir();
            trace!("User-local configuration path: {:?}", user_config_path);
            let physfs = vfs::PhysicalFS::new(user_config_path, false);
            overlay.push_back(Box::new(physfs));
        }

        let fs = Filesystem {
            vfs: overlay,
            resources_path,
            zip_path: resources_zip_path,
            user_config_path: user_config_path.to_path_buf(),
            user_data_path: user_data_path.to_path_buf(),
        };

        Ok(fs)
    }

    /// Opens the given `path` and returns the resulting `File`
    /// in read-only mode.
    pub(crate) fn open<P: AsRef<path::Path>>(&self, path: P) -> GameResult<File> {
        self.vfs.open(path.as_ref()).map(|f| File::VfsFile(f))
    }

    /// Opens a file in the user directory with the given
    /// [`filesystem::OpenOptions`](struct.OpenOptions.html).
    /// Note that even if you open a file read-write, it can only
    /// write to files in the "user" directory.
    pub(crate) fn open_options<P: AsRef<path::Path>>(
        &self,
        path: P,
        options: OpenOptions,
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
    pub(crate) fn create<P: AsRef<path::Path>>(&self, path: P) -> GameResult<File> {
        self.vfs.create(path.as_ref()).map(|f| File::VfsFile(f))
    }

    /// Create an empty directory in the user dir
    /// with the given name.  Any parents to that directory
    /// that do not exist will be created.
    pub(crate) fn create_dir<P: AsRef<path::Path>>(&self, path: P) -> GameResult<()> {
        self.vfs.mkdir(path.as_ref())
    }

    /// Deletes the specified file in the user dir.
    pub(crate) fn delete<P: AsRef<path::Path>>(&self, path: P) -> GameResult<()> {
        self.vfs.rm(path.as_ref())
    }

    /// Deletes the specified directory in the user dir,
    /// and all its contents!
    pub(crate) fn delete_dir<P: AsRef<path::Path>>(&self, path: P) -> GameResult<()> {
        self.vfs.rmrf(path.as_ref())
    }

    /// Check whether a file or directory exists.
    pub(crate) fn exists<P: AsRef<path::Path>>(&self, path: P) -> bool {
        self.vfs.exists(path.as_ref())
    }

    /// Check whether a path points at a file.
    pub(crate) fn is_file<P: AsRef<path::Path>>(&self, path: P) -> bool {
        self.vfs
            .metadata(path.as_ref())
            .map(|m| m.is_file())
            .unwrap_or(false)
    }

    /// Check whether a path points at a directory.
    pub(crate) fn is_dir<P: AsRef<path::Path>>(&self, path: P) -> bool {
        self.vfs
            .metadata(path.as_ref())
            .map(|m| m.is_dir())
            .unwrap_or(false)
    }

    /// Returns a list of all files and directories in the resource directory,
    /// in no particular order.
    ///
    /// Lists the base directory if an empty path is given.
    pub(crate) fn read_dir<P: AsRef<path::Path>>(
        &self,
        path: P,
    ) -> GameResult<Box<dyn Iterator<Item = path::PathBuf>>> {
        let itr = self.vfs.read_dir(path.as_ref())?.map(|fname| {
            fname.expect("Could not read file in read_dir()?  Should never happen, I hope!")
        });
        Ok(Box::new(itr))
    }

    fn write_to_string(&self) -> String {
        use std::fmt::Write;
        let mut s = String::new();
        for vfs in self.vfs.roots() {
            write!(s, "Source {:?}", vfs).expect("Could not write to string; should never happen?");
            match vfs.read_dir(path::Path::new("/")) {
                Ok(files) => {
                    for itm in files {
                        write!(s, "  {:?}", itm)
                            .expect("Could not write to string; should never happen?");
                    }
                }
                Err(e) => write!(s, " Could not read source: {:?}", e)
                    .expect("Could not write to string; should never happen?"),
            }
        }
        s
    }

    /// Prints the contents of all data directories
    /// to standard output.  Useful for debugging.
    pub(crate) fn print_all(&self) {
        println!("{}", self.write_to_string());
    }

    /// Outputs the contents of all data directories,
    /// using the "info" log level of the [`log`](https://docs.rs/log/) crate.
    /// Useful for debugging.
    pub(crate) fn log_all(&self) {
        info!("{}", self.write_to_string());
    }

    /// Adds the given (absolute) path to the list of directories
    /// it will search to look for resources.
    ///
    /// You probably shouldn't use this in the general case, since it is
    /// harder than it looks to make it bulletproof across platforms.
    /// But it can be very nice for debugging and dev purposes, such as
    /// by pushing `$CARGO_MANIFEST_DIR/resources` to it
    pub(crate) fn mount(&mut self, path: &path::Path, readonly: bool) {
        let physfs = vfs::PhysicalFS::new(path, readonly);
        trace!("Mounting new path: {:?}", physfs);
        self.vfs.push_back(Box::new(physfs));
    }

    /// Adds any object that implements Read + Seek as a zip file.
    ///
    /// Note: This is not intended for system files for the same reasons as
    /// for `.mount()`. Rather, it can be used to read zip files from sources
    /// such as `std::io::Cursor::new(includes_bytes!(...))` in order to embed
    /// resources into the game's executable.
    pub(crate) fn add_zip_file<R: io::Read + io::Seek + 'static>(
        &mut self,
        reader: R,
    ) -> GameResult<()> {
        let zipfs = vfs::ZipFS::from_read(reader)?;
        trace!("Adding zip file from reader");
        self.vfs.push_back(Box::new(zipfs));
        Ok(())
    }

    /// Looks for a file named `/conf.toml` in any resource directory and
    /// loads it if it finds it.
    /// If it can't read it for some reason, returns an error.
    pub(crate) fn read_config(&self) -> GameResult<conf::Conf> {
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

    /// Takes a `Conf` object and saves it to the user directory,
    /// overwriting any file already there.
    pub(crate) fn write_config(&self, conf: &conf::Conf) -> GameResult<()> {
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

/// Opens the given path and returns the resulting `File`
/// in read-only mode.
pub fn open<P: AsRef<path::Path>>(ctx: &Context, path: P) -> GameResult<File> {
    ctx.filesystem.open(path)
}

/// Opens a file in the user directory with the given `filesystem::OpenOptions`.
/// Note that even if you open a file read-only, it can only access
/// files in the user directory.
pub fn open_options<P: AsRef<path::Path>>(
    ctx: &Context,
    path: P,
    options: OpenOptions,
) -> GameResult<File> {
    ctx.filesystem.open_options(path, options)
}

/// Creates a new file in the user directory and opens it
/// to be written to, truncating it if it already exists.
pub fn create<P: AsRef<path::Path>>(ctx: &Context, path: P) -> GameResult<File> {
    ctx.filesystem.create(path)
}

/// Create an empty directory in the user dir
/// with the given name.  Any parents to that directory
/// that do not exist will be created.
pub fn create_dir<P: AsRef<path::Path>>(ctx: &Context, path: P) -> GameResult {
    ctx.filesystem.create_dir(path.as_ref())
}

/// Deletes the specified file in the user dir.
pub fn delete<P: AsRef<path::Path>>(ctx: &Context, path: P) -> GameResult {
    ctx.filesystem.delete(path.as_ref())
}

/// Deletes the specified directory in the user dir,
/// and all its contents!
pub fn delete_dir<P: AsRef<path::Path>>(ctx: &Context, path: P) -> GameResult {
    ctx.filesystem.delete_dir(path.as_ref())
}

/// Check whether a file or directory exists.
pub fn exists<P: AsRef<path::Path>>(ctx: &Context, path: P) -> bool {
    ctx.filesystem.exists(path.as_ref())
}

/// Check whether a path points at a file.
pub fn is_file<P: AsRef<path::Path>>(ctx: &Context, path: P) -> bool {
    ctx.filesystem.is_file(path)
}

/// Check whether a path points at a directory.
pub fn is_dir<P: AsRef<path::Path>>(ctx: &Context, path: P) -> bool {
    ctx.filesystem.is_dir(path)
}

/// Return the full path to the user data directory
pub fn user_data_dir(ctx: &Context) -> &path::Path {
    &ctx.filesystem.user_data_path
}

/// Return the full path to the user config directory
pub fn user_config_dir(ctx: &Context) -> &path::Path {
    &ctx.filesystem.user_config_path
}

/// Returns the full path to the resource directory
/// (even if it doesn't exist)
pub fn resources_dir(ctx: &Context) -> &path::Path {
    &ctx.filesystem.resources_path
}

/// Return the full path to the user data directory
pub fn zip_dir(ctx: &Context) -> &path::Path {
    &ctx.filesystem.zip_path
}

/// Returns a list of all files and directories in the resource directory,
/// in no particular order.
///
/// Lists the base directory if an empty path is given.
pub fn read_dir<P: AsRef<path::Path>>(
    ctx: &Context,
    path: P,
) -> GameResult<Box<dyn Iterator<Item = path::PathBuf>>> {
    ctx.filesystem.read_dir(path)
}

/// Prints the contents of all data directories.
/// Useful for debugging.
pub fn print_all(ctx: &Context) {
    ctx.filesystem.print_all()
}

/// Outputs the contents of all data directories,
/// using the "info" log level of the `log` crate.
/// Useful for debugging.
///
/// See the [`logging` example](https://github.com/ggez/ggez/blob/master/examples/eventloop.rs)
/// for how to collect log information.
pub fn log_all(ctx: &Context) {
    ctx.filesystem.log_all()
}

/// Adds the given (absolute) path to the list of directories
/// it will search to look for resources.
///
/// You probably shouldn't use this in the general case, since it is
/// harder than it looks to make it bulletproof across platforms.
/// But it can be very nice for debugging and dev purposes, such as
/// by pushing `$CARGO_MANIFEST_DIR/resources` to it
pub fn mount(ctx: &mut Context, path: &path::Path, readonly: bool) {
    ctx.filesystem.mount(path, readonly)
}

/// Looks for a file named `/conf.toml` in any resource directory and
/// loads it if it finds it.
/// If it can't read it for some reason, returns an error.
pub fn read_config(ctx: &Context) -> GameResult<conf::Conf> {
    ctx.filesystem.read_config()
}

/// Takes a `Conf` object and saves it to the user directory,
/// overwriting any file already there.
pub fn write_config(ctx: &Context, conf: &conf::Conf) -> GameResult {
    ctx.filesystem.write_config(conf)
}

#[cfg(test)]
mod tests {
    use crate::conf;
    use crate::error::*;
    use crate::filesystem::*;
    use std::io::{Read, Write};
    use std::path;

    fn dummy_fs_for_tests() -> Filesystem {
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
    fn headless_test_file_exists() {
        let f = dummy_fs_for_tests();

        let tile_file = path::Path::new("/tile.png");
        assert!(f.exists(tile_file));
        assert!(f.is_file(tile_file));

        let tile_file = path::Path::new("/oglebog.png");
        assert!(!f.exists(tile_file));
        assert!(!f.is_file(tile_file));
        assert!(!f.is_dir(tile_file));
    }

    #[test]
    fn headless_test_read_dir() {
        let f = dummy_fs_for_tests();

        let dir_contents_size = f.read_dir("/").unwrap().count();
        assert!(dir_contents_size > 0);
    }

    #[test]
    fn headless_test_create_delete_file() {
        let fs = dummy_fs_for_tests();
        let test_file = path::Path::new("/testfile.txt");
        let bytes = b"test";

        {
            let mut file = fs.create(test_file).unwrap();
            let _ = file.write(bytes).unwrap();
        }
        {
            let mut buffer = Vec::new();
            let mut file = fs.open(test_file).unwrap();
            let _ = file.read_to_end(&mut buffer).unwrap();
            assert_eq!(bytes, buffer.as_slice());
        }

        fs.delete(test_file).unwrap();
    }

    #[test]
    fn headless_test_file_not_found() {
        let fs = dummy_fs_for_tests();
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
    fn headless_test_write_config() {
        let f = dummy_fs_for_tests();
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
