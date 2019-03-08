use failure;

/// A general way to talk about decompression algorithms. This allows Wasmer to decompress with
/// different decompression methods.
pub trait Decompress {
    fn decompress(compressed_data: Vec<u8>) -> Result<Vec<u8>, failure::Error>;
}

/// [zstd compression](https://facebook.github.io/zstd/)
pub struct ZStdDecompression;

impl Decompress for ZStdDecompression {
    fn decompress(compressed_data: Vec<u8>) -> Result<Vec<u8>, failure::Error> {
        zstd::stream::decode_all(&compressed_data[..])
            .map_err(|e| e.into())
    }
}

pub struct NoDecompression;

impl Decompress for NoDecompression {
    fn decompress(compressed_data: Vec<u8>) -> Result<Vec<u8>, failure::Error> {
        Ok(compressed_data)
    }
}
