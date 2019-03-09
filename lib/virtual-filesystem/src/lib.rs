#[macro_use]
extern crate failure;

use crate::decompress::Decompress;
use std::collections::HashMap;
use std::path::{PathBuf, Path};
use std::io::Read;

pub mod decompress;

pub type DataVec = Vec<Vec<u8>>;
pub type PathMap = HashMap<PathBuf, usize>;

pub struct Vfs {
    data: DataVec,
    paths: PathMap,
}

pub type CompressedArchive = Vec<u8>;

impl Vfs {
    pub fn new<D: Decompress>(compressed_archive: CompressedArchive) -> Result<Self, failure::Error> {
        let decompressed_archive = D::decompress(compressed_archive)?;
        let mut ar = tar::Archive::new(&decompressed_archive[..]);

        let mut data = vec![];
        let mut paths = HashMap::new();

        let entries = ar.entries()?;
        for entry in entries {
            let mut entry = entry?;
            let path = entry.path()?.into_owned();
            let mut file_data = String::new();
            entry.read_to_string(&mut file_data)?;
            // insert the file data into the vec
            let index = data.len();
            data.push(file_data.into_bytes());
            // insert the path into the map
            paths.insert(path, index);
        }

        let vfs = Vfs {
            data,
            paths,
        };

        Ok(vfs)
    }

    pub fn read<P: AsRef<Path>>(&self, path: P) -> Result<Vec<u8>, failure::Error> {
        let read_result = self.paths.get(path.as_ref()).ok_or(VfsError::FileDoesNotExist)?;
        if self.data.len() < *read_result {
            panic!("File data for path in virtual file system does not exist. {}", read_result);
        }
        let data = self.data.get(*read_result).unwrap().clone();
        Ok(data)
    }
}

#[derive(Debug, Fail)]
pub enum VfsError {
    #[fail(display = "File does not exist.")]
    FileDoesNotExist,
}

#[cfg(test)]
mod test {
    use crate::Vfs;
    use crate::decompress::NoDecompression;
    use std::fs::File;
    use std::io::{Read, Write};
    use tempdir;

    #[test]
    fn empty_archive() {
        let empty_archive = vec![];
        let vfs_result = Vfs::new::<NoDecompression>(empty_archive);
        assert!(vfs_result.is_ok(), "Failed to create file system from empty archive");
        let vfs = vfs_result.unwrap();
    }

    #[test]
    fn single_file_archive() {
        // create temp dir with a temp file
        let tmp_dir = tempdir::TempDir::new("single_file_archive").unwrap();
        let file_path = tmp_dir.path().join("foo.txt");
        let mut tmp_file = File::create(file_path.clone()).unwrap();
        writeln!(tmp_file, "foo foo foo").unwrap();
        let mut tar_data = vec![];
        let mut ar = tar::Builder::new(tar_data);
        ar.append_path_with_name(file_path, "foo.txt").unwrap();
        let mut archive = ar.into_inner().unwrap();

        let vfs_result = Vfs::new::<NoDecompression>(archive);
        assert!(vfs_result.is_ok(), "Failed to create file system from empty archive");
        let vfs = vfs_result.unwrap();

        // read the file
        let read_result = vfs.read("foo.txt");
        assert!(read_result.is_ok(), "Failed to read file from vfs");
        let actual_data = read_result.unwrap();
        let expected_data = "foo foo foo\n".as_bytes();
        assert_eq!(actual_data, expected_data, "Contents were not equal");
    }
}
