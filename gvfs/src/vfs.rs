//! A virtual file system layer that lets us define multiple
//! "file systems" with various backing stores, then merge them
//! together.
//!
//! Basically a re-implementation of the C library `PhysFS`.  The
//! `vfs` crate does something similar but has a couple design
//! decisions that make it kind of incompatible with this use case:
//! the relevant trait for it has generic methods so we can't use it
//! as a trait object, and its path abstraction is not the most
//! convenient.

use std::cell::RefCell;
use std::fmt::{self, Debug};
use std::fs;
use std::io::{self, Read, Seek, Write};
use std::path::{self, Path, PathBuf};

use zip;

use crate::{Error, Result};

/// What it says on the tin
fn convenient_path_to_str(path: &path::Path) -> Result<&str> {
    path.to_str().ok_or_else(|| {
        let errmessage = format!("Invalid path format for resource: {:?}", path);
        Error::VfsError(errmessage)
    })
}

/// Our basic trait for files.  All different types of filesystem
/// must provide a thing that implements this trait.
pub trait VFile: Read + Write + Seek + Debug {}

impl<T> VFile for T where T: Read + Write + Seek + Debug {}

/// Options for opening files
///
/// We need our own version of this structure because the one in
/// `std` annoyingly doesn't let you read the read/write/create/etc
/// state out of it.
#[must_use]
#[derive(Debug, Default, Copy, Clone, PartialEq)]
pub struct OpenOptions {
    read: bool,
    write: bool,
    create: bool,
    append: bool,
    truncate: bool,
}

impl OpenOptions {
    /// Create a new instance with defaults.
    pub fn new() -> OpenOptions {
        Default::default()
    }

    /// Open for reading
    pub fn read(mut self, read: bool) -> OpenOptions {
        self.read = read;
        self
    }

    /// Open for writing
    pub fn write(mut self, write: bool) -> OpenOptions {
        self.write = write;
        self
    }

    /// Create the file if it does not exist yet
    pub fn create(mut self, create: bool) -> OpenOptions {
        self.create = create;
        self
    }

    /// Append at the end of the file
    pub fn append(mut self, append: bool) -> OpenOptions {
        self.append = append;
        self
    }

    /// Truncate the file to 0 bytes after opening
    pub fn truncate(mut self, truncate: bool) -> OpenOptions {
        self.truncate = truncate;
        self
    }

    fn to_fs_openoptions(self) -> fs::OpenOptions {
        let mut opt = fs::OpenOptions::new();
        let _ = opt
            .read(self.read)
            .write(self.write)
            .create(self.create)
            .append(self.append)
            .truncate(self.truncate)
            .create(self.create);
        opt
    }
}

/// A trait for a virtual file system, such as a zip file or a point
/// in the real file system.
pub trait VFS: Debug {
    /// Open the file at this path with the given options
    fn open_options(&self, path: &Path, open_options: OpenOptions) -> Result<Box<dyn VFile>>;
    /// Open the file at this path for reading
    fn open(&self, path: &Path) -> Result<Box<dyn VFile>> {
        self.open_options(path, OpenOptions::new().read(true))
    }
    /// Open the file at this path for writing, truncating it if it exists already
    fn create(&self, path: &Path) -> Result<Box<dyn VFile>> {
        self.open_options(
            path,
            OpenOptions::new().write(true).create(true).truncate(true),
        )
    }
    /// Open the file at this path for appending, creating it if necessary
    fn append(&self, path: &Path) -> Result<Box<dyn VFile>> {
        self.open_options(
            path,
            OpenOptions::new().write(true).create(true).append(true),
        )
    }
    /// Create a directory at the location by this path
    fn mkdir(&self, path: &Path) -> Result;

    /// Remove a file or an empty directory.
    fn rm(&self, path: &Path) -> Result;

    /// Remove a file or directory and all its contents
    fn rmrf(&self, path: &Path) -> Result;

    /// Check if the file exists
    fn exists(&self, path: &Path) -> bool;

    /// Get the file's metadata
    fn metadata(&self, path: &Path) -> Result<Box<dyn VMetadata>>;

    /// Retrieve all file and directory entries in the given directory.
    fn read_dir(&self, path: &Path) -> Result<Box<dyn Iterator<Item = Result<PathBuf>>>>;

    /// Retrieve the actual location of the VFS root, if available.
    fn to_path_buf(&self) -> Option<PathBuf>;
}

/// The metadata we can read from a file.
pub trait VMetadata {
    /// Returns whether or not it is a directory.
    /// Note that zip files don't actually have directories, awkwardly,
    /// just files with very long names.
    fn is_dir(&self) -> bool;
    /// Returns whether or not it is a file.
    fn is_file(&self) -> bool;
    /// Returns the length of the thing.  If it is a directory,
    /// the result of this is undefined/platform dependent.
    fn len(&self) -> u64;
}

/// A VFS that points to a directory and uses it as the root of its
/// file hierarchy.
///
/// It IS allowed to have symlinks in it!  They're surprisingly
/// difficult to get rid of.
#[derive(Clone)]
pub struct PhysicalFS {
    root: PathBuf,
    readonly: bool,
}

/// Metadata for a physical file.
#[derive(Debug, Clone)]
pub struct PhysicalMetadata(fs::Metadata);

impl VMetadata for PhysicalMetadata {
    fn is_dir(&self) -> bool {
        self.0.is_dir()
    }
    fn is_file(&self) -> bool {
        self.0.is_file()
    }
    fn len(&self) -> u64 {
        self.0.len()
    }
}

/// This takes an absolute path and returns either a sanitized relative
/// version of it, or None if there's something bad in it.
///
/// What we want is an absolute path with no `..`'s in it, so, something
/// like "/foo" or "/foo/bar.txt".  This means a path with components
/// starting with a `RootDir`, and zero or more `Normal` components.
///
/// We gotta return a new path because there's apparently no real good way
/// to turn an absolute path into a relative path with the same
/// components (other than the first), and pushing an absolute `Path`
/// onto a `PathBuf` just completely nukes its existing contents.
fn sanitize_path(path: &path::Path) -> Option<PathBuf> {
    let mut c = path.components();
    match c.next() {
        Some(path::Component::RootDir) => (),
        _ => return None,
    }

    fn is_normal_component(comp: path::Component) -> Option<&str> {
        match comp {
            path::Component::Normal(s) => s.to_str(),
            _ => None,
        }
    }

    // This could be done more cleverly but meh
    let mut accm = PathBuf::new();
    for component in c {
        if let Some(s) = is_normal_component(component) {
            accm.push(s)
        } else {
            return None;
        }
    }
    Some(accm)
}

impl PhysicalFS {
    /// Create new PhysicalFS
    pub fn new(root: &Path, readonly: bool) -> Self {
        PhysicalFS {
            root: root.into(),
            readonly,
        }
    }

    /// Takes a given absolute `&Path` and returns
    /// a new PathBuf containing the canonical
    /// absolute path you get when appending it
    /// to this filesystem's root.
    ///
    /// So if this FS's root is `/home/icefox/foo` then
    /// calling `fs.to_absolute("/bar")` should return
    /// `/home/icefox/foo/bar`
    fn to_absolute(&self, p: &Path) -> Result<PathBuf> {
        if let Some(safe_path) = sanitize_path(p) {
            let mut root_path = self.root.clone();
            root_path.push(safe_path);
            Ok(root_path)
        } else {
            let msg = format!(
                "Path {:?} is not valid: must be an absolute path with no \
                 references to parent directories",
                p
            );
            Err(Error::VfsError(msg))
        }
    }

    /// Creates the PhysicalFS's root directory if necessary.
    /// Idempotent.
    ///
    /// This way we can avoid creating the directory
    /// until it's actually used, though it IS a tiny bit of a
    /// performance malus.
    fn create_root(&self) -> Result {
        if !self.root.exists() {
            fs::create_dir_all(&self.root).map_err(Error::from)
        } else {
            Ok(())
        }
    }
}

impl Debug for PhysicalFS {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "<PhysicalFS root: {}>", self.root.display())
    }
}

impl VFS for PhysicalFS {
    /// Open the file at this path with the given options
    fn open_options(&self, path: &Path, open_options: OpenOptions) -> Result<Box<dyn VFile>> {
        if self.readonly
            && (open_options.write
                || open_options.create
                || open_options.append
                || open_options.truncate)
        {
            let msg = format!(
                "Cannot alter file {:?} in root {:?}, filesystem read-only",
                path, self
            );
            return Err(Error::VfsError(msg));
        }
        self.create_root()?;
        let p = self.to_absolute(path)?;
        open_options
            .to_fs_openoptions()
            .open(p)
            .map(|x| Box::new(x) as Box<dyn VFile>)
            .map_err(Error::from)
    }

    /// Create a directory at the location by this path
    fn mkdir(&self, path: &Path) -> Result {
        if self.readonly {
            return Err(Error::VfsError(
                "Tried to make directory {} but FS is \
                 read-only"
                    .to_string(),
            ));
        }
        self.create_root()?;
        let p = self.to_absolute(path)?;
        //println!("Creating {:?}", p);
        fs::DirBuilder::new()
            .recursive(true)
            .create(p)
            .map_err(Error::from)
    }

    /// Remove a file
    fn rm(&self, path: &Path) -> Result {
        if self.readonly {
            return Err(Error::VfsError(
                "Tried to remove file {} but FS is read-only".to_string(),
            ));
        }

        self.create_root()?;
        let p = self.to_absolute(path)?;
        if p.is_dir() {
            fs::remove_dir(p).map_err(Error::from)
        } else {
            fs::remove_file(p).map_err(Error::from)
        }
    }

    /// Remove a file or directory and all its contents
    fn rmrf(&self, path: &Path) -> Result {
        if self.readonly {
            return Err(Error::VfsError(
                "Tried to remove file/dir {} but FS is \
                 read-only"
                    .to_string(),
            ));
        }

        self.create_root()?;
        let p = self.to_absolute(path)?;
        if p.is_dir() {
            fs::remove_dir_all(p).map_err(Error::from)
        } else {
            fs::remove_file(p).map_err(Error::from)
        }
    }

    /// Check if the file exists
    fn exists(&self, path: &Path) -> bool {
        match self.to_absolute(path) {
            Ok(p) => p.exists(),
            _ => false,
        }
    }

    /// Get the file's metadata
    fn metadata(&self, path: &Path) -> Result<Box<dyn VMetadata>> {
        self.create_root()?;
        let p = self.to_absolute(path)?;
        p.metadata()
            .map(|m| Box::new(PhysicalMetadata(m)) as Box<dyn VMetadata>)
            .map_err(Error::from)
    }

    /// Retrieve the path entries in this path
    fn read_dir(&self, path: &Path) -> Result<Box<dyn Iterator<Item = Result<PathBuf>>>> {
        self.create_root()?;
        let p = self.to_absolute(path)?;
        // This is inconvenient because path() returns the full absolute
        // path of the bloody file, which is NOT what we want!
        // But if we use file_name() to just get the name then it is ALSO not what we want!
        // what we WANT is the full absolute file path, *relative to the resources dir*.
        // So that we can do read_dir("/foobar/"), and for each file, open it and query
        // it and such by name.
        // So we build the paths ourself.
        let direntry_to_path = |entry: &fs::DirEntry| -> Result<PathBuf> {
            let fname = entry
                .file_name()
                .into_string()
                .expect("Non-unicode char in file path?  Should never happen, I hope!");
            let mut pathbuf = PathBuf::from(path);
            pathbuf.push(fname);
            Ok(pathbuf)
        };
        let itr = fs::read_dir(p)?
            .map(|entry| direntry_to_path(&entry?))
            .collect::<Vec<_>>()
            .into_iter();
        Ok(Box::new(itr))
    }

    /// Retrieve the actual location of the VFS root, if available.
    fn to_path_buf(&self) -> Option<PathBuf> {
        Some(self.root.clone())
    }
}

/// A structure that joins several VFS's together in order.
///
/// So if a file isn't found in one FS it will search through them
/// looking for it and return the
#[derive(Debug)]
pub struct OverlayFS {
    roots: Vec<Box<dyn VFS>>,
}

impl OverlayFS {
    /// New OverlayFS containing zero filesystems.
    pub fn new() -> Self {
        Self { roots: Vec::new() }
    }

    /// Adds a new VFS to the end of the list.
    pub fn push(&mut self, fs: Box<dyn VFS>) {
        self.roots.push(fs);
    }

    /// Get a reference to the inner file systems,
    /// in search order.
    pub fn roots(&self) -> &[Box<dyn VFS>] {
        &self.roots
    }
}

impl VFS for OverlayFS {
    /// Open the file at this path with the given options
    fn open_options(&self, path: &Path, open_options: OpenOptions) -> Result<Box<dyn VFile>> {
        let mut tried: Vec<(PathBuf, Error)> = vec![];

        for vfs in &self.roots {
            match vfs.open_options(path, open_options) {
                Err(e) => {
                    if let Some(vfs_path) = vfs.to_path_buf() {
                        tried.push((vfs_path, e));
                    } else {
                        tried.push((PathBuf::from("<invalid path>"), e));
                    }
                }
                f => return f,
            }
        }
        let errmessage = String::from(convenient_path_to_str(path)?);
        Err(Error::ResourceNotFound(errmessage, tried))
    }

    /// Create a directory at the location by this path
    fn mkdir(&self, path: &Path) -> Result {
        for vfs in &self.roots {
            match vfs.mkdir(path) {
                Err(_) => (),
                f => return f,
            }
        }
        Err(Error::VfsError(format!(
            "Could not find anywhere writeable to make dir {:?}",
            path
        )))
    }

    /// Remove a file
    fn rm(&self, path: &Path) -> Result {
        for vfs in &self.roots {
            match vfs.rm(path) {
                Err(_) => (),
                f => return f,
            }
        }
        Err(Error::VfsError(format!("Could not remove file {:?}", path)))
    }

    /// Remove a file or directory and all its contents
    fn rmrf(&self, path: &Path) -> Result {
        for vfs in &self.roots {
            match vfs.rmrf(path) {
                Err(_) => (),
                f => return f,
            }
        }
        Err(Error::VfsError(format!(
            "Could not remove file/dir {:?}",
            path
        )))
    }

    /// Check if the file exists
    fn exists(&self, path: &Path) -> bool {
        for vfs in &self.roots {
            if vfs.exists(path) {
                return true;
            }
        }

        false
    }

    /// Get the file's metadata
    fn metadata(&self, path: &Path) -> Result<Box<dyn VMetadata>> {
        for vfs in &self.roots {
            match vfs.metadata(path) {
                Err(_) => (),
                f => return f,
            }
        }
        Err(Error::VfsError(format!(
            "Could not get metadata for file/dir {:?}",
            path
        )))
    }

    /// Retrieve the path entries in this path
    fn read_dir(&self, path: &Path) -> Result<Box<dyn Iterator<Item = Result<PathBuf>>>> {
        // This is tricky 'cause we have to actually merge iterators together...
        // Doing it the simple and stupid way works though.
        let mut v = Vec::new();
        for fs in &self.roots {
            if let Ok(rddir) = fs.read_dir(path) {
                v.extend(rddir)
            }
        }
        Ok(Box::new(v.into_iter()))
    }

    /// Retrieve the actual location of the VFS root, if available.
    fn to_path_buf(&self) -> Option<PathBuf> {
        None
    }
}

trait ZipArchiveAccess {
    fn by_name<'a>(&'a mut self, name: &str) -> zip::result::ZipResult<zip::read::ZipFile<'a>>;
    fn by_index<'a>(
        &'a mut self,
        file_number: usize,
    ) -> zip::result::ZipResult<zip::read::ZipFile<'a>>;
    fn len(&self) -> usize;
}

impl<T: Read + Seek> ZipArchiveAccess for zip::ZipArchive<T> {
    fn by_name(&mut self, name: &str) -> zip::result::ZipResult<zip::read::ZipFile> {
        self.by_name(name)
    }

    fn by_index(&mut self, file_number: usize) -> zip::result::ZipResult<zip::read::ZipFile> {
        self.by_index(file_number)
    }

    fn len(&self) -> usize {
        self.len()
    }
}

impl Debug for dyn ZipArchiveAccess {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        // Hide the contents; for an io::Cursor, this would print what is
        // likely to be megabytes of data.
        write!(f, "<ZipArchiveAccess>")
    }
}

/// A filesystem backed by a zip file.
#[derive(Debug)]
pub struct ZipFS {
    // It's... a bit jankity.
    // Zip files aren't really designed to be virtual filesystems,
    // and the structure of the `zip` crate doesn't help.  See the various
    // issues that have been filed on it by icefoxen.
    //
    // ALSO THE SEMANTICS OF ZIPARCHIVE AND HAVING ZIPFILES BORROW IT IS
    // HORRIFICALLY BROKEN BY DESIGN SO WE'RE JUST GONNA REFCELL IT AND COPY
    // ALL CONTENTS OUT OF IT AAAAA.
    source: Option<PathBuf>,
    archive: RefCell<Box<dyn ZipArchiveAccess>>,
    // We keep an index of what files are in the zip file
    // because trying to read it lazily is a pain in the butt.
    index: Vec<String>,
}

impl ZipFS {
    /// Make new VFS from a zip file
    pub fn new(filename: &Path) -> Result<Self> {
        let f = fs::File::open(filename)?;
        let archive = Box::new(zip::ZipArchive::new(f)?);
        ZipFS::from_boxed_archive(archive, Some(filename.into()))
    }

    /// Creates a `ZipFS` from any `Read+Seek` object, most useful with an
    /// in-memory `std::io::Cursor`.
    pub fn from_read<R>(reader: R) -> Result<Self>
    where
        R: Read + Seek + 'static,
    {
        let archive = Box::new(zip::ZipArchive::new(reader)?);
        ZipFS::from_boxed_archive(archive, None)
    }

    fn from_boxed_archive(
        mut archive: Box<dyn ZipArchiveAccess>,
        source: Option<PathBuf>,
    ) -> Result<Self> {
        let idx = (0..archive.len())
            .map(|i| {
                archive
                    .by_index(i)
                    .expect("Should never happen!")
                    .name()
                    .to_string()
            })
            .collect();
        Ok(Self {
            source,
            archive: RefCell::new(archive),
            index: idx,
        })
    }
}

/// A wrapper to contain a zipfile so we can implement
/// (janky) Seek on it and such.
///
/// We're going to do it the *really* janky way and just read
/// the whole `ZipFile` into a buffer, which is kind of awful but means
/// we don't have to deal with lifetimes, self-borrowing structs,
/// rental, re-implementing Seek on compressed data, making multiple zip
/// zip file objects share a single file handle, or any of that
/// other nonsense.
#[derive(Clone)]
pub struct ZipFileWrapper {
    buffer: io::Cursor<Vec<u8>>,
}

impl ZipFileWrapper {
    fn new(z: &mut zip::read::ZipFile) -> Result<Self> {
        let mut b = Vec::new();
        let _ = z.read_to_end(&mut b)?;
        Ok(Self {
            buffer: io::Cursor::new(b),
        })
    }
}

impl io::Read for ZipFileWrapper {
    fn read(&mut self, buf: &mut [u8]) -> io::Result<usize> {
        self.buffer.read(buf)
    }
}

impl io::Write for ZipFileWrapper {
    fn write(&mut self, _buf: &[u8]) -> io::Result<usize> {
        panic!("Cannot write to a zip file!")
    }

    fn flush(&mut self) -> io::Result<()> {
        Ok(())
    }
}

impl io::Seek for ZipFileWrapper {
    fn seek(&mut self, pos: io::SeekFrom) -> io::Result<u64> {
        self.buffer.seek(pos)
    }
}

impl Debug for ZipFileWrapper {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "<Zipfile>")
    }
}

#[derive(Debug, Copy, Clone, PartialEq)]
struct ZipMetadata {
    len: u64,
    is_dir: bool,
    is_file: bool,
}

impl ZipMetadata {
    /// Returns a ZipMetadata, or None if the file does not exist or such.
    /// This is not QUITE correct; since zip archives don't actually have
    /// directories (just long filenames), we can't get a directory's metadata
    /// this way without basically just faking it.
    ///
    /// This does make listing a directory rather screwy.
    fn new(name: &str, archive: &mut dyn ZipArchiveAccess) -> Option<Self> {
        match archive.by_name(name) {
            Err(_) => None,
            Ok(zipfile) => {
                let len = zipfile.size();
                Some(ZipMetadata {
                    len,
                    is_file: true,
                    is_dir: false, // mu
                })
            }
        }
    }
}

impl VMetadata for ZipMetadata {
    fn is_dir(&self) -> bool {
        self.is_dir
    }
    fn is_file(&self) -> bool {
        self.is_file
    }
    fn len(&self) -> u64 {
        self.len
    }
}

impl VFS for ZipFS {
    fn open_options(&self, path: &Path, open_options: OpenOptions) -> Result<Box<dyn VFile>> {
        // Zip is readonly
        let path = convenient_path_to_str(path)?;
        if open_options.write || open_options.create || open_options.append || open_options.truncate
        {
            let msg = format!(
                "Cannot alter file {:?} in zipfile {:?}, filesystem read-only",
                path, self
            );
            return Err(Error::VfsError(msg));
        }
        let mut stupid_archive_borrow = self.archive
            .try_borrow_mut()
            .expect("Couldn't borrow ZipArchive in ZipFS::open_options(); should never happen! Report a bug at https://github.com/ggez/gvfs/");
        let mut f = stupid_archive_borrow.by_name(path)?;
        let zipfile = ZipFileWrapper::new(&mut f)?;
        Ok(Box::new(zipfile) as Box<dyn VFile>)
    }

    fn mkdir(&self, path: &Path) -> Result {
        let msg = format!(
            "Cannot mkdir {:?} in zipfile {:?}, filesystem read-only",
            path, self
        );
        Err(Error::VfsError(msg))
    }

    fn rm(&self, path: &Path) -> Result {
        let msg = format!(
            "Cannot rm {:?} in zipfile {:?}, filesystem read-only",
            path, self
        );
        Err(Error::VfsError(msg))
    }

    fn rmrf(&self, path: &Path) -> Result {
        let msg = format!(
            "Cannot rmrf {:?} in zipfile {:?}, filesystem read-only",
            path, self
        );
        Err(Error::VfsError(msg))
    }

    fn exists(&self, path: &Path) -> bool {
        let mut stupid_archive_borrow = self.archive
            .try_borrow_mut()
            .expect("Couldn't borrow ZipArchive in ZipFS::exists(); should never happen!  Report a bug at https://github.com/ggez/gvfs/");
        if let Ok(path) = convenient_path_to_str(path) {
            stupid_archive_borrow.by_name(path).is_ok()
        } else {
            false
        }
    }

    fn metadata(&self, path: &Path) -> Result<Box<dyn VMetadata>> {
        let path = convenient_path_to_str(path)?;
        let mut stupid_archive_borrow = self.archive
            .try_borrow_mut()
            .expect("Couldn't borrow ZipArchive in ZipFS::metadata(); should never happen! Report a bug at https://github.com/ggez/gvfs/");
        match ZipMetadata::new(path, &mut **stupid_archive_borrow) {
            None => Err(Error::VfsError(format!(
                "Metadata not found in zip file for {}",
                path
            ))),
            Some(md) => Ok(Box::new(md) as Box<dyn VMetadata>),
        }
    }

    /// Zip files don't have real directories, so we (incorrectly) hack it by
    /// just looking for a path prefix for now.
    fn read_dir(&self, path: &Path) -> Result<Box<dyn Iterator<Item = Result<PathBuf>>>> {
        let path = convenient_path_to_str(path)?;
        let itr = self
            .index
            .iter()
            .filter(|s| s.starts_with(path))
            .map(|s| Ok(PathBuf::from(s)))
            .collect::<Vec<_>>();
        Ok(Box::new(itr.into_iter()))
    }

    fn to_path_buf(&self) -> Option<PathBuf> {
        self.source.clone()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::{self, BufRead};

    #[test]
    fn headless_test_path_filtering() {
        // Valid paths
        let p = path::Path::new("/foo");
        assert!(sanitize_path(p).is_some());

        let p = path::Path::new("/foo/");
        assert!(sanitize_path(p).is_some());

        let p = path::Path::new("/foo/bar.txt");
        assert!(sanitize_path(p).is_some());

        let p = path::Path::new("/");
        assert!(sanitize_path(p).is_some());

        // Invalid paths
        let p = path::Path::new("../foo");
        assert!(sanitize_path(p).is_none());

        let p = path::Path::new("foo");
        assert!(sanitize_path(p).is_none());

        let p = path::Path::new("/foo/../../");
        assert!(sanitize_path(p).is_none());

        let p = path::Path::new("/foo/../bop");
        assert!(sanitize_path(p).is_none());

        let p = path::Path::new("/../bar");
        assert!(sanitize_path(p).is_none());

        let p = path::Path::new("");
        assert!(sanitize_path(p).is_none());
    }

    #[test]
    fn headless_test_read() {
        let cargo_path = Path::new(env!("CARGO_MANIFEST_DIR"));
        let fs = PhysicalFS::new(cargo_path, true);
        let f = fs.open(Path::new("/Cargo.toml")).unwrap();
        let mut bf = io::BufReader::new(f);
        let mut s = String::new();
        let _ = bf.read_line(&mut s).unwrap();
        // Trim whitespace from string 'cause it will
        // potentially be different on Windows and Unix.
        let trimmed_string = s.trim();
        assert_eq!(trimmed_string, "[package]");
    }

    #[test]
    fn headless_test_read_overlay() {
        let cargo_path = Path::new(env!("CARGO_MANIFEST_DIR"));
        let fs1 = PhysicalFS::new(cargo_path, true);
        let mut f2path = PathBuf::from(cargo_path);
        f2path.push("src");
        let fs2 = PhysicalFS::new(&f2path, true);
        let mut ofs = OverlayFS::new();
        ofs.push(Box::new(fs1));
        ofs.push(Box::new(fs2));

        assert!(ofs.exists(Path::new("/Cargo.toml")));
        assert!(ofs.exists(Path::new("/lib.rs")));
        assert!(!ofs.exists(Path::new("/foobaz.rs")));
    }

    #[test]
    fn headless_test_physical_all() {
        let cargo_path = Path::new(env!("CARGO_MANIFEST_DIR"));
        let fs = PhysicalFS::new(cargo_path, false);
        let testdir = Path::new("/testdir");
        let f1 = Path::new("/testdir/file1.txt");

        // Delete testdir if it is still lying around
        if fs.exists(testdir) {
            fs.rmrf(testdir).unwrap();
        }
        assert!(!fs.exists(testdir));

        // Create and delete test dir
        fs.mkdir(testdir).unwrap();
        assert!(fs.exists(testdir));
        fs.rm(testdir).unwrap();
        assert!(!fs.exists(testdir));

        let test_string = "Foo!";
        fs.mkdir(testdir).unwrap();
        {
            let mut f = fs.append(f1).unwrap();
            let _ = f.write(test_string.as_bytes()).unwrap();
        }
        {
            let mut buf = Vec::new();
            let mut f = fs.open(f1).unwrap();
            let _ = f.read_to_end(&mut buf).unwrap();
            assert_eq!(&buf[..], test_string.as_bytes());
        }

        {
            // Test metadata()
            let m = fs.metadata(f1).unwrap();
            assert!(m.is_file());
            assert!(!m.is_dir());
            assert_eq!(m.len(), 4);

            let m = fs.metadata(testdir).unwrap();
            assert!(!m.is_file());
            assert!(m.is_dir());
            // Not exactly sure what the "length" of a directory is, buuuuuut...
            // It appears to vary based on the platform in fact.
            // On my desktop, it's 18.
            // On Travis's VM, it's 4096.
            // On Appveyor's VM, it's 0.
            // So, it's meaningless.
            //assert_eq!(m.len(), 18);
        }

        {
            // Test read_dir()
            let r = fs.read_dir(testdir).unwrap();
            assert_eq!(r.count(), 1);
            let r = fs.read_dir(testdir).unwrap();
            for f in r {
                let fname = f.unwrap();
                assert!(fs.exists(&fname));
            }
        }

        {
            assert!(fs.exists(f1));
            fs.rm(f1).unwrap();
            assert!(!fs.exists(f1));
        }

        fs.rmrf(testdir).unwrap();
        assert!(!fs.exists(testdir));
    }

    fn make_zip_fs() -> Box<dyn VFS> {
        let mut finished_zip_bytes: io::Cursor<_> = {
            let zip_bytes = io::Cursor::new(vec![]);
            let mut zip_archive = zip::ZipWriter::new(zip_bytes);

            zip_archive
                .start_file("fake_file_name.txt", zip::write::FileOptions::default())
                .unwrap();
            let _bytes = zip_archive.write(b"Zip contents!").unwrap();
            zip_archive.add_directory("fake_dir", zip::write::FileOptions::default())
                .unwrap();
            zip_archive
                .start_file("fake_dir/file.txt", zip::write::FileOptions::default())
                .unwrap();
            let _bytes = zip_archive.write(b"Zip contents!").unwrap();

            zip_archive.finish().unwrap()
        };

        let _bytes = finished_zip_bytes.seek(io::SeekFrom::Start(0)).unwrap();
        let zfs = ZipFS::from_read(finished_zip_bytes).unwrap();
        Box::new(zfs)
    }

    #[test]
    fn test_zip_files() {
        let zfs = make_zip_fs();

        assert!(zfs.exists(Path::new("fake_file_name.txt".into())));

        let mut contents = String::new();
        let _bytes = zfs
            .open(Path::new("fake_file_name.txt"))
            .unwrap()
            .read_to_string(&mut contents)
            .unwrap();
        assert_eq!(contents, "Zip contents!");
    }

    #[test]
    fn headless_test_zip_all() {
        let fs = make_zip_fs();
        let testdir = Path::new("/testdir");
        let testfile = Path::new("/file1.txt");
        // TODO: Fix absolute vs. relative paths for zip files...
        let existing_file = Path::new("fake_file_name.txt");
        let existing_dir = Path::new("fake_dir");

        assert!(!fs.exists(testfile));
        assert!(!fs.exists(testdir));
        assert!(fs.exists(existing_file));
        // TODO: This fails, why?
        //assert!(fs.exists(existing_dir));


        // Create and delete test dir -- which always fails
        assert!(fs.mkdir(testdir).is_err());
        assert!(!fs.exists(testdir));
        assert!(fs.rm(testdir).is_err());

        // Reading an existing file succeeds.
        let _ = fs.open(existing_file).unwrap();
        // Writing to a new fails
        assert!(fs.create(testfile).is_err());
        // Appending a file fails
        assert!(fs.append(testfile).is_err());

        {
            // Test metadata()
            let m = fs.metadata(existing_file).unwrap();
            assert!(m.is_file());
            assert!(!m.is_dir());
            assert_eq!(m.len(), 13);

            // TODO: Fix
            /*
            let m = fs.metadata(existing_dir).unwrap();
            assert!(!m.is_file());
            assert!(m.is_dir());
*/

            assert!(fs.metadata(testfile).is_err());
        }

        {
            // TODO: Test read_dir()
            /*
            let r = fs.read_dir(existing_dir).unwrap();
            assert_eq!(r.count(), 1);
            let r = fs.read_dir(testdir).unwrap();
            for f in r {
                let fname = f.unwrap();
                assert!(fs.exists(&fname));
            }
             */
        }

        assert!(fs.rmrf(testdir).is_err());
        assert!(fs.rmrf(existing_dir).is_err());

    }

    // BUGGO: TODO: Make sure all functions are tested for OverlayFS and ZipFS!!
}
