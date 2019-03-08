use crate::decompress::ZStdDecompression;
use crate::decompress::Decompress;
use std::collections::HashMap;
use std::path::PathBuf;

pub mod decompress;

pub type DataVec = Vec<Vec<u8>>;
pub type PathMap = HashMap<PathBuf, usize>;

pub struct Vfs {
    data: DataVec,
    paths: PathMap,
}

pub type CompressedArchive = Vec<u8>;

impl Vfs {
    pub fn new(compressed_archive: CompressedArchive) -> Result<Self, failure::Error> {
        let decompressed_archive = ZStdDecompression::decompress(compressed_archive)?;
        let ar = tar::Archive::new(&decompressed_archive[..]);
        // uncompress and unarchive
        // push records into a vec, the indexes represent the keys

        unimplemented!()
    }
}
