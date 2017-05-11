use std::sync::Arc;
use std::path::{self, PathBuf};
use std::fs;
use std::fmt::{self, Debug};
use std::io::{self, Read, Write, Seek};

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
    fn open_with_options(&self, path: &Path, open_options: &OpenOptions) -> GameResult<Box<VFile>>;
    /// Open the file at this path for reading
    fn open(&self, path: &Path) -> GameResult<Box<VFile>> {
        self.open_with_options(path, OpenOptions::new().read(true))
    }
    /// Open the file at this path for writing, truncating it if it exists already
    fn create(&self, path: &Path) -> GameResult<Box<VFile>> {
        self.open_with_options(path, OpenOptions::new().write(true).create(true).truncate(true))
    }
    /// Open the file at this path for appending, creating it if necessary
    fn append(&self, path: &Path) -> GameResult<Box<VFile>> {
        self.open_with_options(path, OpenOptions::new().write(true).create(true).append(true))
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

    /// Retrieve the path entries in this path
    fn read_dir(&self, path: &Path) -> GameResult<Box<Iterator<Item = GameResult<Box<Path>>>>>;
}

pub trait VMetadata {}

/// A VFS that points to a directory and uses it as the root of its
/// file heirarchy.
///
/// It IS allowed to have symlinks in it!  For now.
pub struct PhysicalFS {
    root: Arc<PathBuf>,
    readonly: bool,
}

pub struct PhysicalMetadata(fs::Metadata);

impl VMetadata for PhysicalMetadata {}


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
    fn new(root: &str, readonly: bool) -> Self {
        PhysicalFS {
            root: Arc::new(root.into()),
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
            let mut root_path = (*self.root).clone();
            root_path.push(safe_path);
            println!("Path is {:?}", root_path);
            Ok(root_path)
        } else {
            let msg = format!("Path {:?} is not valid: must be an absolute path with no references to parent directories", p);
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
    fn open_with_options(&self, path: &Path, open_options: &OpenOptions) -> GameResult<Box<VFile>> {
        if self.readonly {
            if open_options.write || open_options.create || open_options.append || open_options.truncate {
                let msg = format!("Cannot alter file {:?} in root {:?}, filesystem read-only", path, self);
                return Err(GameError::FilesystemError(msg));
            }
        }
        let p = self.get_absolute(path)?;
        open_options.to_fs_openoptions().open(p)
            .map(|x| Box::new(x) as Box<VFile>)
            .map_err(GameError::from)
    }
    
    /// Create a directory at the location by this path
    fn mkdir(&self, path: &Path) -> GameResult<()> {
        if self.readonly {
            return Err(GameError::FilesystemError("Tried to make directory {} but FS is read-only".to_string()));
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
            return Err(GameError::FilesystemError("Tried to remove file {} but FS is read-only".to_string()));
        }

        let p = self.get_absolute(path)?;
        if p.is_dir() {
            fs::remove_dir(p)
                .map_err(GameError::from)
        } else {
            fs::remove_file(p)
                .map_err(GameError::from)
        }
    }

    /// Remove a file or directory and all its contents
    fn rmrf(&self, path: &Path) -> GameResult<()> {
        if self.readonly {
            return Err(GameError::FilesystemError("Tried to remove file/dir {} but FS is read-only".to_string()));
        }
        
        let p = self.get_absolute(path)?;
        if p.is_dir() {
            fs::remove_dir_all(p)
                .map_err(GameError::from)
        } else {
            fs::remove_file(p)
                .map_err(GameError::from)
        }
    }

    /// Check if the file exists
    fn exists(&self, path: &Path) -> bool {
        match self.get_absolute(path) {
            Ok(p) => p.exists(),
            _ => false
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
    fn read_dir(&self, path: &Path) -> GameResult<Box<Iterator<Item = GameResult<Box<Path>>>>> {
        // TODO
        Err(GameError::FilesystemError("Foo!".to_string()))
    }
}

/// A structure that joins several VFS's together in order.
/// VecDeque instead of Vec?
#[derive(Debug)]
struct OverlayFS {
    roots: Vec<Box<VFS>>,
}

impl OverlayFS {
    fn new() -> Self {
        Self {
            roots: Vec::new()
        }
    }

    /// Adds a new VFS to the end of the list.
    fn push(&mut self, fs: Box<VFS>) {
        &self.roots.push(fs);
    }
}

impl VFS for OverlayFS {
    /// Open the file at this path with the given options
    fn open_with_options(&self, path: &Path, open_options: &OpenOptions) -> GameResult<Box<VFile>> {
        for vfs in &self.roots {
            match vfs.open_with_options(path, open_options) {
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
        Err(GameError::FilesystemError(format!("Could not find anywhere writeable to make dir {}", path)))
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
    fn read_dir(&self, path: &Path) -> GameResult<Box<Iterator<Item = GameResult<Box<Path>>>>> {
        // TODO: This is tricky 'cause we have to actually merge iterators together...
        Err(GameError::FilesystemError("Foo!".to_string()))
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
        ofs.push(Box::new(fs1));
        ofs.push(Box::new(fs2));
        
        assert!(ofs.exists("/Cargo.toml"));
        assert!(ofs.exists("/lib.rs"));
        assert!(!ofs.exists("/foobaz.rs"));
    }

    #[test]
    fn test_physical_all() {
        let fs = PhysicalFS::new(env!("CARGO_MANIFEST_DIR"), false);
        let testdir = "/testdir";
        let f1 = "/testdir/file1.txt";
        let f2 = "/testdir/file2.txt";
        
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
