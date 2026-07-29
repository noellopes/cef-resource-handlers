use crate::{CustomResourceHandlerFactory, ResourceHandlerError};
use once_cell::sync::Lazy;
use std::fs::File;
use std::io::{Read, Seek, SeekFrom};
use std::path::{Path, PathBuf};

static EXE_DIR: Lazy<Option<PathBuf>> = Lazy::new(|| {
    std::env::current_exe()
        .ok()
        .and_then(|path| path.parent().map(|p| p.to_path_buf()))
});

/// A `ContentProvider` implementation that serves content from local files on the filesystem.
/// It resolves file paths relative to the directory of the executable.
pub struct LocalFileContentProvider {
    file: File,
    file_size: Option<usize>,
    mime_type: String,
}

impl LocalFileContentProvider {
    fn full_path(file_path: &str) -> PathBuf {
        if let Some(exe_dir) = &*EXE_DIR {
            return exe_dir.join(file_path);
        }

        PathBuf::from(file_path)
    }

    fn file_size(file_path: &Path) -> Option<usize> {
        std::fs::metadata(file_path)
            .ok()
            .map(|metadata| metadata.len() as usize)
    }

    fn from_file_path(file_path: &Path) -> Result<Self, std::io::Error> {
        let file_size = Self::file_size(file_path);
        let file = File::open(file_path)?;
        let mime_type = mime_guess::from_path(file_path)
            .first_or_octet_stream()
            .to_string();

        Ok(Self {
            file,
            file_size,
            mime_type,
        })
    }
}

impl crate::ContentProvider for LocalFileContentProvider {
    fn from_request(request_info: &crate::RequestInfo) -> Result<Self, ResourceHandlerError> {
        let file_path = Self::full_path(&request_info.path);
        Self::from_file_path(&file_path)
            .map_err(|error| ResourceHandlerError::OpenFileError(file_path, error.to_string()))
    }

    fn size(&self) -> Option<usize> {
        self.file_size
    }

    fn mime_type(&self) -> &str {
        &self.mime_type
    }

    fn should_cache(&self) -> bool {
        true
    }

    fn read(
        &mut self,
        data_out: *mut u8,
        offset: usize,
        bytes_to_read: usize,
    ) -> Result<usize, ResourceHandlerError> {
        self.file.seek(SeekFrom::Start(offset as u64))?;
        let slice = unsafe { std::slice::from_raw_parts_mut(data_out, bytes_to_read) };
        Ok(self.file.read(slice)?)
    }
}

/// A factory type for creating `LocalFileContentProvider` instances as resource handlers.
pub type LocalFileResourceHandlerFactory = CustomResourceHandlerFactory<LocalFileContentProvider>;
