use std::sync::Arc;
use std::path;
use std::fs;
use std::fmt::{self, Debug};
use std::io::{self, Read, Write, Seek};

use {GameResult, GameError};

pub type Path = str;
/*
pub struct VFile {
    handle: fs::File,
}
*/


pub trait VFile: Read + Write + Seek + Debug {}

impl<T> VFile for T where T: Read + Write + Seek + Debug {}

/*
/// Options for opening files
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
}
*/

use std::borrow::Cow;
use std::path::PathBuf;


pub trait VFS: Debug {
    /// Open the file at this path with the given options
    fn open_with_options(&self, path: &Path, openOptions: &fs::OpenOptions) -> GameResult<Box<VFile>>;
    /// Open the file at this path for reading
    fn open(&self, path: &Path) -> GameResult<Box<VFile>> {
        self.open_with_options(path, fs::OpenOptions::new().read(true))
    }
    /// Open the file at this path for writing, truncating it if it exists already
    fn create(&self, path: &Path) -> GameResult<Box<VFile>> {
        self.open_with_options(path, fs::OpenOptions::new().write(true).create(true).truncate(true))
    }
    /// Open the file at this path for appending, creating it if necessary
    fn append(&self, path: &Path) -> GameResult<Box<VFile>> {
        self.open_with_options(path, fs::OpenOptions::new().write(true).create(true).append(true))
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
pub struct PhysicalFS {
    root: Arc<PathBuf>,
    readonly: bool,
}

pub struct PhysicalMetadata(fs::Metadata);

impl VMetadata for PhysicalMetadata {}



/// Helper function to turn a path::Component into an Option<String> iff the Component
/// is a normal portion.
///
/// Basically this is to help turn a canonicalized absolute path into a relative path.
fn component_filter(comp: path::Component) -> Option<String> {
    match comp {
        path::Component::Normal(osstr) => Some(osstr.to_string_lossy().into_owned()),
        _ => None
    }
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
        let pathbuf = PathBuf::from(p);
        let relative_path = pathbuf.components().filter_map(component_filter);
        let mut full_path = (*self.root).clone();
        full_path.extend(relative_path);
        full_path.canonicalize()?;
        if !full_path.starts_with(&*self.root) {
            panic!("Tried to create an AltPath that exits the AltrootFS's root dir");
        }
        Ok(full_path)
    }
}

impl Debug for PhysicalFS {
    fn fmt(&self, f: &mut fmt::Formatter) -> Result<(), fmt::Error> {
        write!(f, "<PhysicalFS root: {}>", self.root.display())
    }
}


impl VFS for PhysicalFS {
    /// Open the file at this path with the given options
    /// XXX: TODO: Check for read-only
    fn open_with_options(&self, path: &Path, openOptions: &fs::OpenOptions) -> GameResult<Box<VFile>> {
        let p = self.get_absolute(path)?;
        openOptions.open(p)
            .map(|x| Box::new(x) as Box<VFile>)
            .map_err(GameError::from)
    }
    
    /// Create a directory at the location by this path
    fn mkdir(&self, path: &Path) -> GameResult<()> {
        if self.readonly {
            return Err(GameError::FilesystemError("Tried to make directory {} but FS is read-only".to_string()));
        }
        let p = self.get_absolute(path)?;
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
    fn open_with_options(&self, path: &Path, openOptions: &fs::OpenOptions) -> GameResult<Box<VFile>> {
        for vfs in &self.roots {
            match vfs.open_with_options(path, openOptions) {
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
    fn test_read() {
        let fs = PhysicalFS::new(env!("CARGO_MANIFEST_DIR"), true);
        let f = fs.open("Cargo.toml").unwrap();
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
        
        assert!(ofs.exists("Cargo.toml"));
        assert!(ofs.exists("lib.rs"));
        assert!(!ofs.exists("foobaz.rs"));
    }

}
