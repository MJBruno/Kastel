//! État runtime des flux de fichiers tamponnés de `std.io`.
//!
//! Ces objets ne stockent aucune `Value` : le GC n'a donc pas de graphe enfant
//! à parcourir. La fermeture explicite du writer propage les erreurs de flush.

use std::fs::{File, OpenOptions};
use std::io::{self, BufReader, BufWriter, Write};
use std::path::{Path, PathBuf};
use std::sync::atomic::{AtomicUsize, Ordering};

pub const DEFAULT_BUFFER_CAPACITY: usize = 8 * 1024;
pub const MAX_BUFFER_CAPACITY: usize = 16 * 1024 * 1024;
pub const MAX_READ_SIZE: usize = 16 * 1024 * 1024;

static NEXT_BUFFERED_READER_ID: AtomicUsize = AtomicUsize::new(1);
static NEXT_BUFFERED_WRITER_ID: AtomicUsize = AtomicUsize::new(1);

fn validate_capacity(capacity: usize) -> io::Result<()> {
    if capacity == 0 || capacity > MAX_BUFFER_CAPACITY {
        return Err(io::Error::new(
            io::ErrorKind::InvalidInput,
            format!("buffer capacity must be between 1 and {MAX_BUFFER_CAPACITY} bytes"),
        ));
    }
    Ok(())
}

/// Lecteur tamponné propriétaire de son fichier.
#[derive(Debug)]
pub struct BufReaderState {
    id: usize,
    pub(crate) reader: Option<BufReader<File>>,
    pub(crate) path: PathBuf,
    capacity: usize,
}

impl BufReaderState {
    pub fn open(path: &Path, capacity: usize) -> io::Result<Self> {
        validate_capacity(capacity)?;
        let file = File::open(path)?;
        Ok(Self::new(path.to_path_buf(), file, capacity))
    }

    fn new(path: PathBuf, file: File, capacity: usize) -> Self {
        // Les constructeurs externes valident la capacité avant d'appeler new.
        Self {
            id: NEXT_BUFFERED_READER_ID.fetch_add(1, Ordering::Relaxed),
            reader: Some(BufReader::with_capacity(capacity, file)),
            path,
            capacity,
        }
    }

    pub fn id(&self) -> usize {
        self.id
    }

    pub fn capacity(&self) -> usize {
        self.capacity
    }

    pub fn is_closed(&self) -> bool {
        self.reader.is_none()
    }

    /// Ferme le flux. Retourne `true` seulement lors de la première fermeture.
    pub fn close(&mut self) -> bool {
        self.reader.take().is_some()
    }
}

impl PartialEq for BufReaderState {
    fn eq(&self, other: &Self) -> bool {
        self.id == other.id
    }
}

/// Écrivain tamponné propriétaire de son fichier.
#[derive(Debug)]
pub struct BufWriterState {
    id: usize,
    pub(crate) writer: Option<BufWriter<File>>,
    pub(crate) path: PathBuf,
    capacity: usize,
}

impl BufWriterState {
    /// Ouvre en écriture en créant le fichier et en tronquant son contenu existant.
    pub fn create(path: &Path, capacity: usize) -> io::Result<Self> {
        validate_capacity(capacity)?;
        let file = OpenOptions::new()
            .write(true)
            .create(true)
            .truncate(true)
            .open(path)?;
        Ok(Self::new(path.to_path_buf(), file, capacity))
    }

    /// Ouvre en écriture à la fin du fichier, sans supprimer son contenu.
    /// Le fichier est créé s'il n'existe pas.
    pub fn append(path: &Path, capacity: usize) -> io::Result<Self> {
        validate_capacity(capacity)?;
        let file = OpenOptions::new().append(true).create(true).open(path)?;
        Ok(Self::new(path.to_path_buf(), file, capacity))
    }

    fn new(path: PathBuf, file: File, capacity: usize) -> Self {
        Self {
            id: NEXT_BUFFERED_WRITER_ID.fetch_add(1, Ordering::Relaxed),
            writer: Some(BufWriter::with_capacity(capacity, file)),
            path,
            capacity,
        }
    }

    pub fn id(&self) -> usize {
        self.id
    }

    pub fn capacity(&self) -> usize {
        self.capacity
    }

    pub fn is_closed(&self) -> bool {
        self.writer.is_none()
    }

    /// Vide le buffer avant de libérer le fichier. En cas d'erreur, le writer
    /// reste ouvert afin que l'appelant puisse réessayer `flush` ou `close`.
    pub fn close(&mut self) -> io::Result<bool> {
        let Some(writer) = self.writer.as_mut() else {
            return Ok(false);
        };
        writer.flush()?;
        self.writer.take();
        Ok(true)
    }
}

impl PartialEq for BufWriterState {
    fn eq(&self, other: &Self) -> bool {
        self.id == other.id
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::env;
    use std::time::{SystemTime, UNIX_EPOCH};

    fn temp_path(label: &str) -> PathBuf {
        let stamp = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .expect("system clock should be after the Unix epoch")
            .as_nanos();
        env::temp_dir().join(format!("kastel-buffered-{label}-{}-{stamp}.tmp", std::process::id()))
    }

    #[test]
    fn buffered_reader_owns_and_closes_its_file() {
        let path = temp_path("reader");
        std::fs::write(&path, b"hello\nworld").unwrap();
        let mut reader = BufReaderState::open(&path, DEFAULT_BUFFER_CAPACITY).unwrap();
        assert_eq!(reader.capacity(), DEFAULT_BUFFER_CAPACITY);
        assert!(!reader.is_closed());
        assert!(reader.close());
        assert!(!reader.close());
        assert!(reader.is_closed());
        std::fs::remove_file(path).unwrap();
    }

    #[test]
    fn buffered_writer_flushes_before_close() {
        let path = temp_path("writer");
        let mut writer = BufWriterState::create(&path, 1024).unwrap();
        {
            let handle = writer.writer.as_mut().unwrap();
            handle.write_all(b"persisted").unwrap();
        }
        writer.close().unwrap();
        assert_eq!(std::fs::read(&path).unwrap(), b"persisted");
        assert!(writer.is_closed());
        std::fs::remove_file(path).unwrap();
    }

    #[test]
    fn buffered_writer_append_preserves_existing_contents() {
        let path = temp_path("append");
        std::fs::write(&path, b"existing").unwrap();
        let mut writer = BufWriterState::append(&path, 64).unwrap();
        writer.writer.as_mut().unwrap().write_all(b"-added").unwrap();
        writer.close().unwrap();
        assert_eq!(std::fs::read(&path).unwrap(), b"existing-added");
        std::fs::remove_file(path).unwrap();
    }

    #[test]
    fn buffer_capacity_has_a_safe_nonzero_bound() {
        assert!(validate_capacity(1).is_ok());
        assert!(validate_capacity(MAX_BUFFER_CAPACITY).is_ok());
        assert!(validate_capacity(0).is_err());
        assert!(validate_capacity(MAX_BUFFER_CAPACITY + 1).is_err());
    }
}
