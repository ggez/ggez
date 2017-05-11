use std::sync::Arc;
use std::path;

use {GameResult, GameError};

pub type Path = str;

pub struct VFile {
}

pub struct VMetadata {
}
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

use std::borrow::Cow;
use std::path::PathBuf;

pub trait VFS {
    /// Open the file at this path with the given options
    fn open_with_options(&self, path: &Path, openOptions: &OpenOptions) -> GameResult<Box<VFile>>;
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

    /// Remove a file
    fn rm(&self, path: &Path) -> GameResult<()>;

    /// Remove a file or directory and all its contents
    fn rmrf(&self, path: &Path) -> GameResult<()>;


    /// The file name of this path
    fn file_name(&self, path: &Path) -> Option<String>;

    /// The extension of this filename
    fn extension(&self, path: &Path) -> Option<String>;

    /// append a segment to this path
    fn resolve(&self, path: &Path, path: &String) -> String;

    /// Get the parent path
    fn parent(&self, path: &Path) -> Option<String>;

    /// Check if the file existst
    fn exists(&self, path: &Path) -> bool;

    /// Get the file's metadata
    fn metadata(&self, path: &Path) -> GameResult<Box<VMetadata>>;

    /// Retrieve the path entries in this path
    fn read_dir(&self, path: &Path) -> GameResult<Box<Iterator<Item = GameResult<Box<Path>>>>>;

    /// Retrieve a string representation
    fn to_string(&self, path: &Path) -> String;

    /// Retrieve a standard &PathBuf, if available (usually only for PhysicalFS)
    fn to_path_buf(&self, path: &Path) -> Option<&PathBuf>;
}

struct VfsList {
    roots: Vec<Box<VFS>>,
}

/// A VFS that points to a directory and uses it as the root of its
/// file heirarchy.
pub struct PhysicalFS {
    root: Arc<PathBuf>,
}

impl VFS for PhysicalFS {
    /// Open the file at this path with the given options
    fn open_with_options(&self, path: &Path, openOptions: &OpenOptions) -> GameResult<Box<VFile>> {
        Err(GameError::FilesystemError("Foo!".to_string()))
    }
    
    /// Create a directory at the location by this path
    fn mkdir(&self, path: &Path) -> GameResult<()> {
        Ok(())
    }

    /// Remove a file
    fn rm(&self, path: &Path) -> GameResult<()> {
        Ok(())
    }

    /// Remove a file or directory and all its contents
    fn rmrf(&self, path: &Path) -> GameResult<()> {
        Ok(())
    }


    /// The file name of this path
    fn file_name(&self, path: &Path) -> Option<String> {
        None
    }

    /// The extension of this filename
    fn extension(&self, path: &Path) -> Option<String> {
        None
    }

    /// append a segment to this path
    fn resolve(&self, path: &Path, path_other: &String) -> String {
        "".to_string()
    }

    /// Get the parent path
    fn parent(&self, path: &Path) -> Option<String> {
        None
    }

    /// Check if the file existst
    fn exists(&self, path: &Path) -> bool {
        false
    }

    /// Get the file's metadata
    fn metadata(&self, path: &Path) -> GameResult<Box<VMetadata>> {
        Err(GameError::FilesystemError("Foo!".to_string()))
    }

    /// Retrieve the path entries in this path
    fn read_dir(&self, path: &Path) -> GameResult<Box<Iterator<Item = GameResult<Box<Path>>>>> {
        Err(GameError::FilesystemError("Foo!".to_string()))
    }

    /// Retrieve a string representation
    fn to_string(&self, path: &Path) -> String {
        "".to_string()
    }

    /// Retrieve a standard &PathBuf, if available (usually only for PhysicalFS)
    fn to_path_buf(&self, path: &Path) -> Option<&PathBuf> {
        None
    }

}

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
    fn new(root: &str) -> Self {
        PhysicalFS {
            root: Arc::new(root.into()),
        }
    }

    /// Takes a given path (&str) and returns
    /// a new PathBuf containing the canonical
    /// absolute path you get when appending it
    /// to this filesystem's root.
    fn get_absolute(&self, p: &str) -> PathBuf {
        let pathbuf = PathBuf::from(p);
        let relative_path = pathbuf.components().filter_map(component_filter);
        let mut full_path = (*self.root).clone();
        full_path.extend(relative_path);
        full_path.canonicalize().unwrap();
        if !full_path.starts_with(&*self.root) {
            panic!("Tried to create an AltPath that exits the AltrootFS's root dir");
        }
        full_path
    }
}

#[cfg(test)]
mod tests {
    #[test]
    fn test_test() {
        assert!(true);
    }
}
