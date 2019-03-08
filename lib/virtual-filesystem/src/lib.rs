use crate::decompress::ZStdDecompression;
use crate::decompress::Decompress;
use std::collections::HashMap;
use std::path::PathBuf;
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
            let mut file_data = vec![];
            entry.read(&mut file_data)?;
            // insert the file data into the vec
            data.push(file_data);
            let index = data.len();
            // insert the path into the map
            paths.insert(path, index);
        }

        let vfs = Vfs {
            data,
            paths,
        };

        Ok(vfs)
    }
}
