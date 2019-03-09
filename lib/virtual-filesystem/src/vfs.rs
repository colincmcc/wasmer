use std::collections::HashMap;
use std::path::{PathBuf, Path};
use crate::decompress::Decompress;
use std::io::Read;
use std::sync::Arc;
use std::sync::atomic::{AtomicUsize, Ordering};

pub type Fd = i32;

pub type File = (PathBuf, usize);
pub type DataVec = Vec<Vec<u8>>;
pub type PathMap = HashMap<PathBuf, usize>;
pub type FdMap = HashMap<Fd, File>;
pub type PathFdsMap = HashMap<PathBuf, Vec<Fd>>;

/// The structure holding all the data for the virtual filesystem.
pub struct Vfs {
    /// A collection of raw data blobs representing all file data in the virtual filesystem.
    data: DataVec,
    /// A Map of paths to ~~indexes of Vfs::data~~ file descriptors.
    /// Use the file descriptors map to look up file data.
    paths: PathMap,
    /// A Map of file descriptors to indexes of Vfs::data.
    /// All file descriptors offer read-only permissions...for now.
    fds: FdMap,
    /// A map of paths mapped to list of file descriptors. This gets updated when
    path_fds: PathFdsMap,
    /// Counter for file descriptors
    fd_count: Arc<AtomicUsize>,
}

pub type CompressedArchive = Vec<u8>;

impl Vfs {
    pub fn new<D: Decompress>(compressed_archive: CompressedArchive) -> Result<Self, failure::Error> {
        let decompressed_archive = D::decompress(compressed_archive)?;
        let mut ar = tar::Archive::new(&decompressed_archive[..]);

        let mut data = vec![];
        let mut paths = HashMap::new();
        let mut path_fds = HashMap::new();

        // TODO: populate with important file descriptors like std streams
        let mut fds = HashMap::new();

        let fd_count: usize = 0;

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
            fds,
            path_fds,
            fd_count: Arc::new(AtomicUsize::new(fd_count)),
        };

        Ok(vfs)
    }

    fn create_file_descriptor(&mut self) -> Fd {
        self.fd_count.fetch_add(1, Ordering::Monotonic)
    }

    /// Like fs::read, will read a file from the virtual filesystem.
    pub fn read<P: AsRef<Path>>(&self, path: P) -> Result<Vec<u8>, failure::Error> {
        let read_result = self.paths.get(path.as_ref()).ok_or(VfsError::FileDoesNotExist)?;
        if self.data.len() < *read_result {
            panic!("File data for path in virtual file system does not exist. {}", read_result);
        }
        let data = self.data.get(*read_result).unwrap().clone();
        Ok(data)
    }

    /// Returns a new file descriptor for the file
    pub fn open<P: AsRef<Path>>(&mut self, path: P) -> Result<Fd, failure::Error> {
        // if the path exists in the file system, create a new file descriptor
        let new_fd = self.create_file_descriptor();
        // associate the file descriptor with the path and the data

        let get_fd_result = self.paths.get(path.as_ref());
        match get_fd_result {
            Some(fd) => { // there is an existing file descriptor
                Ok(fd)
            },
            None => { // no fd, so create a new one
                let fd: usize = self.fd_count.fetch_add(1, Ordering::Monotonic);
                Ok(fd)
//                unimplemented!()
            }
        }
//        if self.data.len() < *read_result {
//            panic!("File data for path in virtual file system does not exist. {}", read_result);
//        }
//        let data = self.data.get(*read_result).unwrap().clone();
        Ok(0)
    }
}

#[derive(Debug, Fail)]
pub enum VfsError {
    #[fail(display = "File does not exist.")]
    FileDoesNotExist,
}

#[cfg(test)]
mod open_test {
    use std::fs::File;
    use crate::decompress::NoDecompression;
    use std::io::{Write};
    use crate::vfs::Vfs;

    #[test]
    fn open_file_non_existent_file() {
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

        // open the file, get a file descriptor
//        let open_result = vfs.open();
//        assert!(open_result.is_ok(), "Failed to open file in the virtual filesystem.");
    }
}

#[cfg(test)]
mod read_test {
    use crate::decompress::NoDecompression;
    use std::fs::File;
    use std::io::{Read, Write};
    use tempdir;
    use crate::vfs::Vfs;

    #[test]
    fn empty_archive() {
        let empty_archive = vec![];
        let vfs_result =  Vfs::new::<NoDecompression>(empty_archive);
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

    #[test]
    fn two_files_in_archive() {
        // create temp dir with a temp file
        let tmp_dir = tempdir::TempDir::new("two_files_in_archive").unwrap();
        let foo_file_path = tmp_dir.path().join("foo.txt");
        let bar_file_path = tmp_dir.path().join("bar.txt");

        let mut foo_tmp_file = File::create(foo_file_path.clone()).unwrap();
        let mut bar_tmp_file = File::create(bar_file_path.clone()).unwrap();

        writeln!(foo_tmp_file, "foo foo foo").unwrap();
        writeln!(bar_tmp_file, "bar bar").unwrap();

        let mut tar_data = vec![];
        let mut ar = tar::Builder::new(tar_data);
        ar.append_path_with_name(foo_file_path, "foo.txt").unwrap();
        ar.append_path_with_name(bar_file_path, "bar.txt").unwrap();
        let mut archive = ar.into_inner().unwrap();

        let vfs_result = Vfs::new::<NoDecompression>(archive);
        assert!(vfs_result.is_ok(), "Failed to create file system from empty archive");
        let vfs = vfs_result.unwrap();

        // read the file
        let foo_read_result = vfs.read("foo.txt");
        let bar_read_result = vfs.read("bar.txt");

        assert!(foo_read_result.is_ok(), "Failed to read foo.txt from vfs");
        assert!(bar_read_result.is_ok(), "Failed to read bar.txt from vfs");

        let foo_actual_data = foo_read_result.unwrap();
        let bar_actual_data = bar_read_result.unwrap();

        let foo_expected_data = "foo foo foo\n".as_bytes();
        let bar_expected_data = "bar bar\n".as_bytes();

        assert_eq!(foo_actual_data, foo_expected_data, "Contents of `foo.txt` is not correct");
        assert_eq!(bar_actual_data, bar_expected_data, "Contents of `bar.txt` is not correct");
    }
}
