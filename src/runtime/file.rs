//! État runtime des handles de fichiers Kastel.
//!
//! Les handles sont possédés par la VM et ne contiennent aucune `Value` :
//! ils n'ajoutent donc aucun enfant au graphe suivi par le GC.

use std::fs::{File, OpenOptions};
use std::io;
use std::path::{Path, PathBuf};
use std::sync::atomic::{AtomicUsize, Ordering};

static NEXT_FILE_ID: AtomicUsize = AtomicUsize::new(1);
static NEXT_OPTIONS_ID: AtomicUsize = AtomicUsize::new(1);

fn next_file_id() -> usize {
    NEXT_FILE_ID.fetch_add(1, Ordering::Relaxed)
}

fn next_options_id() -> usize {
    NEXT_OPTIONS_ID.fetch_add(1, Ordering::Relaxed)
}

/// Handle OS d'un fichier ouvert.
#[derive(Debug)]
pub struct FileState {
    id: usize,
    pub(crate) file: Option<File>,
    pub(crate) path: PathBuf,
    pub(crate) readable: bool,
    pub(crate) writable: bool,
    pub(crate) append: bool,
}

impl FileState {
    pub fn new(
        path: PathBuf,
        file: File,
        readable: bool,
        writable: bool,
        append: bool,
    ) -> Self {
        Self {
            id: next_file_id(),
            file: Some(file),
            path,
            readable,
            writable,
            append,
        }
    }

    pub fn id(&self) -> usize {
        self.id
    }

    pub fn is_closed(&self) -> bool {
        self.file.is_none()
    }

    /// Fermeture idempotente. Le drop du handle OS ferme aussi le fichier.
    pub fn close(&mut self) -> bool {
        self.file.take().is_some()
    }

    pub fn try_clone(&self) -> io::Result<Self> {
        let file = self.file.as_ref().ok_or_else(|| {
            io::Error::new(io::ErrorKind::NotConnected, "file handle is closed")
        })?;
        let cloned = file.try_clone()?;
        Ok(Self::new(
            self.path.clone(),
            cloned,
            self.readable,
            self.writable,
            self.append,
        ))
    }
}

impl PartialEq for FileState {
    fn eq(&self, other: &Self) -> bool {
        self.id == other.id
    }
}

/// Configuration mutable équivalente à `std::fs::OpenOptions`.
#[derive(Debug)]
pub struct OpenOptionsState {
    id: usize,
    pub(crate) read: bool,
    pub(crate) write: bool,
    pub(crate) append: bool,
    pub(crate) truncate: bool,
    pub(crate) create: bool,
    pub(crate) create_new: bool,
}

impl OpenOptionsState {
    pub fn new() -> Self {
        Self {
            id: next_options_id(),
            read: false,
            write: false,
            append: false,
            truncate: false,
            create: false,
            create_new: false,
        }
    }

    pub fn id(&self) -> usize {
        self.id
    }

    pub fn set(&mut self, option: &str, enabled: bool) -> bool {
        let target = match option {
            "read" => &mut self.read,
            "write" => &mut self.write,
            "append" => &mut self.append,
            "truncate" => &mut self.truncate,
            "create" => &mut self.create,
            "create_new" => &mut self.create_new,
            _ => return false,
        };
        *target = enabled;
        true
    }

    pub fn open(&self, path: &Path) -> io::Result<FileState> {
        let mut options = OpenOptions::new();
        options
            .read(self.read)
            .write(self.write)
            .append(self.append)
            .truncate(self.truncate)
            .create(self.create)
            .create_new(self.create_new);

        let file = options.open(path)?;
        Ok(FileState::new(
            path.to_path_buf(),
            file,
            self.read,
            self.write || self.append,
            self.append,
        ))
    }
}

impl Default for OpenOptionsState {
    fn default() -> Self {
        Self::new()
    }
}

impl PartialEq for OpenOptionsState {
    fn eq(&self, other: &Self) -> bool {
        self.id == other.id
    }
}
