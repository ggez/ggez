//! A virtual file system layer that lets us define multiple
//! "file systems" with various backing stores, then merge them 
//! together.
//!
//! Basically a re-implementation of PhysFS.  The `vfs` crate
//! does something similar but has a couple design decisions that make 
//! it kind of incompatible with this use case;
//!
//! We make some simplifying assumptions as well, namely that
//! Path == str.  Because doing defining all our paths to be 
//! generic `T: Into<Path>` or such means that we can't use the
//! resulting traits as trait objects.

use std::cell::RefCell;
use std::collections::VecDeque;
use std::path::{self, PathBuf};
use std::fs;
use std::fmt::{self, Debug};
use std::io::{self, Read, Write, Seek};

use zip;

use {GameResult, GameError};

pub type Path = str;

pub trait VFile: Read + Write + Seek + Debug {}

impl<T> VFile for T where T: Read + Write + Seek + Debug {}


/// Options for opening files
///
/// We need our own version of this structure because the one in
/// std annoyingly doesn't let you get data out of it.
#[derive(Debug, Default)]
pub struct OpenOptions {
    read: bool,
    write: bool,
    create: bool,
    append: bool,
    truncate: bool,
}

impl OpenOptions {
    /// Create a new instance
    pub fn new() -> OpenOptions {
        Default::default()
    }

    /// Open for reading
    pub fn read(&mut self, read: bool) -> &mut OpenOptions {
        self.read = read;
        self
    }

    /// Open for writing
    pub fn write(&mut self, write: bool) -> &mut OpenOptions {
        self.write = write;
        self
    }

    /// Create the file if it does not exist yet
    pub fn create(&mut self, create: bool) -> &mut OpenOptions {
        self.create = create;
        self
    }

    /// Append at the end of the file
    pub fn append(&mut self, append: bool) -> &mut OpenOptions {
        self.append = append;
        self
    }

    /// Truncate the file to 0 bytes after opening
    pub fn truncate(&mut self, truncate: bool) -> &mut OpenOptions {
        self.truncate = truncate;
        self
    }

    fn to_fs_openoptions(&self) -> fs::OpenOptions {
        let mut opt = fs::OpenOptions::new();
        opt.read(self.read)
            .write(self.write)
            .create(self.create)
            .append(self.append)
            .truncate(self.truncate)
            .create(self.create);
        opt
    }
}

pub trait VFS: Debug {
    /// Open the file at this path with the given options
    fn open_options(&self, path: &Path, open_options: &OpenOptions) -> GameResult<Box<VFile>>;
    /// Open the file at this path for reading
    fn open(&self, path: &Path) -> GameResult<Box<VFile>> {
        self.open_options(path, OpenOptions::new().read(true))
    }
    /// Open the file at this path for writing, truncating it if it exists already
    fn create(&self, path: &Path) -> GameResult<Box<VFile>> {
        self.open_options(path,
                          OpenOptions::new()
                              .write(true)
                              .create(true)
                              .truncate(true))
    }
    /// Open the file at this path for appending, creating it if necessary
    fn append(&self, path: &Path) -> GameResult<Box<VFile>> {
        self.open_options(path,
                          OpenOptions::new().write(true).create(true).append(true))
    }
    /// Create a directory at the location by this path
    fn mkdir(&self, path: &Path) -> GameResult<()>;

    /// Remove a file or an empty directory.
    fn rm(&self, path: &Path) -> GameResult<()>;

    /// Remove a file or directory and all its contents
    fn rmrf(&self, path: &Path) -> GameResult<()>;

    /// Check if the file exists
    fn exists(&self, path: &Path) -> bool;

    /// Get the file's metadata
    fn metadata(&self, path: &Path) -> GameResult<Box<VMetadata>>;

    /// Retrieve all file and directory entries in the given directory.
    fn read_dir(&self, path: &Path) -> GameResult<Box<Iterator<Item = GameResult<PathBuf>>>>;
}

pub trait VMetadata {
    fn is_dir(&self) -> bool;
    fn is_file(&self) -> bool;
    fn len(&self) -> u64;
}

/// A VFS that points to a directory and uses it as the root of its
/// file heirarchy.
///
/// It IS allowed to have symlinks in it!  For now.
pub struct PhysicalFS {
    root: PathBuf,
    readonly: bool,
}

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
/// starting with a RootDir, and zero or more Normal components.
///
/// We gotta return a new path because there's apparently no real good way
/// to turn an absolute path into a relative path with the same
/// components (other than the first), and pushing an absolute Path
/// onto a PathBuf just completely nukes its existing contents.
/// Thanks, Obama.
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
    pub fn new(root: &str, readonly: bool) -> Self {
        PhysicalFS {
            root: root.into(),
            readonly: readonly,
        }
    }

    /// Takes a given path (&str) and returns
    /// a new PathBuf containing the canonical
    /// absolute path you get when appending it
    /// to this filesystem's root.
    fn get_absolute(&self, p: &str) -> GameResult<PathBuf> {
        let p = path::Path::new(p);
        if let Some(safe_path) = sanitize_path(p) {
            let mut root_path = self.root.clone();
            root_path.push(safe_path);
            Ok(root_path)
        } else {
            let msg = format!("Path {:?} is not valid: must be an absolute path with no \
                               references to parent directories",
                              p);
            Err(GameError::FilesystemError(msg))
        }
    }
}

impl Debug for PhysicalFS {
    fn fmt(&self, f: &mut fmt::Formatter) -> Result<(), fmt::Error> {
        write!(f, "<PhysicalFS root: {}>", self.root.display())
    }
}


impl VFS for PhysicalFS {
    /// Open the file at this path with the given options
    fn open_options(&self, path: &Path, open_options: &OpenOptions) -> GameResult<Box<VFile>> {
        if self.readonly {
            if open_options.write || open_options.create || open_options.append ||
               open_options.truncate {
                let msg = format!("Cannot alter file {:?} in root {:?}, filesystem read-only",
                                  path,
                                  self);
                return Err(GameError::FilesystemError(msg));
            }
        }
        let p = self.get_absolute(path)?;
        open_options
            .to_fs_openoptions()
            .open(p)
            .map(|x| Box::new(x) as Box<VFile>)
            .map_err(GameError::from)
    }

    /// Create a directory at the location by this path
    fn mkdir(&self, path: &Path) -> GameResult<()> {
        if self.readonly {
            return Err(GameError::FilesystemError("Tried to make directory {} but FS is \
                                                   read-only"
                                                          .to_string()));
        }
        let p = self.get_absolute(path)?;
        //println!("Creating {:?}", p);
        fs::DirBuilder::new()
            .recursive(true)
            .create(p)
            .map_err(GameError::from)
    }

    /// Remove a file
    fn rm(&self, path: &Path) -> GameResult<()> {
        if self.readonly {
            return Err(GameError::FilesystemError("Tried to remove file {} but FS is read-only"
                                                      .to_string()));
        }

        let p = self.get_absolute(path)?;
        if p.is_dir() {
            fs::remove_dir(p).map_err(GameError::from)
        } else {
            fs::remove_file(p).map_err(GameError::from)
        }
    }

    /// Remove a file or directory and all its contents
    fn rmrf(&self, path: &Path) -> GameResult<()> {
        if self.readonly {
            return Err(GameError::FilesystemError("Tried to remove file/dir {} but FS is \
                                                   read-only"
                                                          .to_string()));
        }

        let p = self.get_absolute(path)?;
        if p.is_dir() {
            fs::remove_dir_all(p).map_err(GameError::from)
        } else {
            fs::remove_file(p).map_err(GameError::from)
        }
    }

    /// Check if the file exists
    fn exists(&self, path: &Path) -> bool {
        match self.get_absolute(path) {
            Ok(p) => p.exists(),
            _ => false,
        }
    }

    /// Get the file's metadata
    fn metadata(&self, path: &Path) -> GameResult<Box<VMetadata>> {
        let p = self.get_absolute(path)?;
        p.metadata()
            .map(|m| Box::new(PhysicalMetadata(m)) as Box<VMetadata>)
            .map_err(GameError::from)
    }

    /// Retrieve the path entries in this path
    fn read_dir(&self, path: &Path) -> GameResult<Box<Iterator<Item = GameResult<PathBuf>>>> {
        let p = self.get_absolute(path)?;
        // BUGGO: This is WRONG because it returns the full absolute
        // path of the bloody file, which is NOT what we want!
        let itr = fs::read_dir(p)?
            .map(|direntry| Ok(PathBuf::from(direntry?.file_name().into_string().unwrap())));
        Ok(Box::new(itr))
    }
}

/// A structure that joins several VFS's together in order.
/// VecDeque instead of Vec?
#[derive(Debug)]
pub struct OverlayFS {
    roots: VecDeque<Box<VFS>>,
}

impl OverlayFS {
    pub fn new() -> Self {
        Self { roots: VecDeque::new() }
    }


    /// Adds a new VFS to the front of the list.
    pub fn push_front(&mut self, fs: Box<VFS>) {
        &self.roots.push_front(fs);
    }

    /// Adds a new VFS to the end of the list.
    pub fn push_back(&mut self, fs: Box<VFS>) {
        &self.roots.push_back(fs);
    }
}

impl VFS for OverlayFS {
    /// Open the file at this path with the given options
    fn open_options(&self, path: &Path, open_options: &OpenOptions) -> GameResult<Box<VFile>> {
        for vfs in &self.roots {
            match vfs.open_options(path, open_options) {
                Err(_) => (),
                f => return f,
            }
        }
        Err(GameError::FilesystemError(format!("File {} not found", path)))
    }

    /// Create a directory at the location by this path
    fn mkdir(&self, path: &Path) -> GameResult<()> {
        for vfs in &self.roots {
            match vfs.mkdir(path) {
                Err(_) => (),
                f => return f,
            }
        }
        Err(GameError::FilesystemError(format!("Could not find anywhere writeable to make dir {}",
                                               path)))
    }

    /// Remove a file
    fn rm(&self, path: &Path) -> GameResult<()> {
        for vfs in &self.roots {
            match vfs.rm(path) {
                Err(_) => (),
                f => return f,
            }
        }
        Err(GameError::FilesystemError(format!("Could not remove file {}", path)))
    }

    /// Remove a file or directory and all its contents
    fn rmrf(&self, path: &Path) -> GameResult<()> {
        for vfs in &self.roots {
            match vfs.rmrf(path) {
                Err(_) => (),
                f => return f,
            }
        }
        Err(GameError::FilesystemError(format!("Could not remove file/dir {}", path)))
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
    fn metadata(&self, path: &Path) -> GameResult<Box<VMetadata>> {
        for vfs in &self.roots {
            match vfs.metadata(path) {
                Err(_) => (),
                f => return f,
            }
        }
        Err(GameError::FilesystemError(format!("Could not remove file/dir {}", path)))
    }

    /// Retrieve the path entries in this path
    fn read_dir(&self, path: &Path) -> GameResult<Box<Iterator<Item = GameResult<PathBuf>>>> {
        // BUGGO: TODO: This is tricky 'cause we have to actually merge iterators together...
        // Err(GameError::FilesystemError("Foo!".to_string()))
        let itr = self.roots.iter()
            .flat_map(|fs| fs.read_dir(path).unwrap()).collect::<Vec<_>>();
        Ok(Box::new(itr.into_iter()))

    }
}

/// A filesystem backed by a zip file.
/// It's... probably going to be a bit jankity.
/// Zip files aren't really designed to be virtual filesystems,
/// and the structure of the `zip` crate doesn't help.  See the various
/// issues that have been filed on it by icefoxen.
///
/// ALSO THE SEMANTICS OF ZIPARCHIVE AND HAVING ZIPFILES BORROW IT IS
/// HORRIFICALLY BROKEN BY DESIGN SO WE'RE JUST GONNA REFCELL IT AND COPY
/// ALL CONTENTS OUT OF IT.
#[derive(Debug)]
pub struct ZipFS {
    source: String,
    archive: RefCell<zip::ZipArchive<fs::File>>,
    // We keep an index of what files are in the zip file
    // because trying to read it lazily is a pain in the butt.
    index: Vec<String>,
}

impl ZipFS {
    pub fn new(filename: &str) -> Self {
        let f = fs::File::open(filename).unwrap();
        let mut archive = zip::ZipArchive::new(f).unwrap();
        let idx = (0..archive.len())
            .map(|i| archive.by_index(i).unwrap().name().to_string())
            .collect();
        Self {
            source: filename.to_string(),
            archive: RefCell::new(archive),
            index: idx,
        }
    }
}

/// A wrapper to contain a zipfile so we can implement
/// (janky) Seek on it and such.
///
/// BUGGO: We're going to do it the *really* janky way and just read
/// the whole ZipFile into a buffer, which is kind of awful but means
/// we don't have to deal with lifetimes, self-borrowing structs,
/// rental, re-implementing Seek on compressed data, or any of that
/// other nonsense.
pub struct ZipFileWrapper {
    //zipfile: zip::read::ZipFile<'a>,
    buffer: io::Cursor<Vec<u8>>,
}

impl ZipFileWrapper {
    fn new(z: &mut zip::read::ZipFile) -> Self {
        let mut b = Vec::new();
        z.read_to_end(&mut b).unwrap();
        Self { buffer: io::Cursor::new(b) }
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
    fn fmt(&self, f: &mut fmt::Formatter) -> Result<(), fmt::Error> {
        write!(f, "<Zipfile>")
    }
}


struct ZipMetadata {
    len: u64,
    is_dir: bool,
    is_file: bool,
}

impl ZipMetadata {
    /// Returns a ZipMetadata, or None if the file does not exist or such.
    /// This is not QUITE correct; since zip archives don't actually have
    /// directories (just long filenames), we can't get a directory's metadata
    /// this way.
    ///
    /// This does make listing a directory
    fn new<T>(name: &str, archive: &mut zip::ZipArchive<T>) -> Option<Self>
        where T: io::Read + io::Seek
    {
        match archive.by_name(name) {
            Err(_) => None,
            Ok(zipfile) => {
                let len = zipfile.size();
                Some(ZipMetadata {
                         len: len,
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
    fn open_options(&self, path: &Path, open_options: &OpenOptions) -> GameResult<Box<VFile>> {
        // Zip is readonly
        if open_options.write || open_options.create || open_options.append ||
           open_options.truncate {
            let msg = format!("Cannot alter file {:?} in zipfile {:?}, filesystem read-only",
                              path,
                              &self.source);
            return Err(GameError::FilesystemError(msg));
        }
        let mut stupid_archive_borrow = self.archive
            .try_borrow_mut()
            .expect("Couldn't borrow ZipArchive in ZipFS::open_options(); should never happen! Report a bug at https://github.com/ggez/ggez/");
        let mut f = stupid_archive_borrow.by_name(path)?;
        Ok(Box::new(ZipFileWrapper::new(&mut f)) as Box<VFile>)
    }

    fn mkdir(&self, path: &Path) -> GameResult<()> {
        let msg = format!("Cannot mkdir {:?} in zipfile {:?}, filesystem read-only",
                          path,
                          &self.source);
        return Err(GameError::FilesystemError(msg));

    }

    fn rm(&self, path: &Path) -> GameResult<()> {
        let msg = format!("Cannot rm {:?} in zipfile {:?}, filesystem read-only",
                          path,
                          &self.source);
        return Err(GameError::FilesystemError(msg));
    }

    fn rmrf(&self, path: &Path) -> GameResult<()> {
        let msg = format!("Cannot rmrf {:?} in zipfile {:?}, filesystem read-only",
                          path,
                          &self.source);
        return Err(GameError::FilesystemError(msg));
    }

    fn exists(&self, path: &Path) -> bool {
        let mut stupid_archive_borrow = self.archive
            .try_borrow_mut()
            .expect("Couldn't borrow ZipArchive in ZipFS::exists(); should never happen!  Report a bug at https://github.com/ggez/ggez/");
        stupid_archive_borrow.by_name(path).is_ok()
    }

    fn metadata(&self, path: &Path) -> GameResult<Box<VMetadata>> {
        let mut stupid_archive_borrow =
            self.archive
                .try_borrow_mut()
                .expect("Couldn't borrow ZipArchive in ZipFS::metadata(); should never happen! Report a bug at https://github.com/ggez/ggez/");
        match ZipMetadata::new(path, &mut stupid_archive_borrow) {
            None => {
                Err(GameError::FilesystemError(format!("Metadata not found in zip file for {}",
                                                       path)))
            }
            Some(md) => Ok(Box::new(md) as Box<VMetadata>),
        }
    }

    fn read_dir(&self, path: &Path) -> GameResult<Box<Iterator<Item = GameResult<PathBuf>>>> {
        // let mut stupid_archive_borrow = self.archive
        //     .try_borrow_mut()
        //     .expect("Couldn't borrow ZipArchive in ZipFS::read_dir(); should never happen!  Report a bug at https://github.com/ggez/ggez/");

        // let itr = //(0..stupid_archive_borrow.len())
        //     (0..1)
        //     .map(|i| stupid_archive_borrow.by_index(i).unwrap())
        //     .filter(|zipfile| zipfile.name().starts_with(path))
        //     .map(|zipfile| Ok(PathBuf::from(zipfile.name())))
        //     .collect::<Vec<_>>();
        // Ok(Box::new(itr.into_iter()))
        // unimplemented!()
        let itr = self.index.iter()
            .filter(|s| s.starts_with(path))
            .map(|s| Ok(PathBuf::from(s)))
            .collect::<Vec<_>>();
        Ok(Box::new(itr.into_iter()))
    }
}

#[cfg(test)]
mod tests {
    use std::io::{self, BufRead};
    use super::*;

    #[test]
    fn test_path_filtering() {
        let p = path::Path::new("/foo");
        sanitize_path(p).unwrap();

        let p = path::Path::new("/foo/");
        sanitize_path(p).unwrap();

        let p = path::Path::new("/foo/bar.txt");
        sanitize_path(p).unwrap();

        let p = path::Path::new("/");
        sanitize_path(p).unwrap();

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
    fn test_read() {
        let fs = PhysicalFS::new(env!("CARGO_MANIFEST_DIR"), true);
        let f = fs.open("/Cargo.toml").unwrap();
        let mut bf = io::BufReader::new(f);
        let mut s = String::new();
        bf.read_line(&mut s).unwrap();
        assert_eq!(&s, "[package]\n");
    }

    #[test]
    fn test_read_overlay() {
        let fs1 = PhysicalFS::new(env!("CARGO_MANIFEST_DIR"), true);
        let mut f2path = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
        f2path.push("src");
        let fs2 = PhysicalFS::new(f2path.to_str().unwrap(), true);
        let mut ofs = OverlayFS::new();
        ofs.push_back(Box::new(fs1));
        ofs.push_back(Box::new(fs2));

        assert!(ofs.exists("/Cargo.toml"));
        assert!(ofs.exists("/lib.rs"));
        assert!(!ofs.exists("/foobaz.rs"));
    }

    #[test]
    fn test_physical_all() {
        let fs = PhysicalFS::new(env!("CARGO_MANIFEST_DIR"), false);
        let testdir = "/testdir";
        let f1 = "/testdir/file1.txt";

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
            f.write(test_string.as_bytes()).unwrap();
        }
        {
            let mut buf = Vec::new();
            let mut f = fs.open(f1).unwrap();
            f.read_to_end(&mut buf).unwrap();
            assert_eq!(&buf[..], test_string.as_bytes());
        }

        // BUGGO: TODO: Make sure all functions are tested for PhysicalFS!
        fs.rmrf(testdir).unwrap();
        assert!(!fs.exists(testdir));
    }
    // BUGGO: TODO: Make sure all functions are tested for OverlayFS too!
    // BUGGO: TODO: Implement ZipFS!
}
