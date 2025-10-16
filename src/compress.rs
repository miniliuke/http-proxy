// Copyright 2025 Cloudflare, Inc.
//
// Licensed under the Apache License, Version 2.0 (the "License");
// you may not use this file except in compliance with the License.
// You may obtain a copy of the License at
//
// http://www.apache.org/licenses/LICENSE-2.0
//
// Unless required by applicable law or agreed to in writing, software
// distributed under the License is distributed on an "AS IS" BASIS,
// WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
// See the License for the specific language governing permissions and
// limitations under the License.

use bytes::Bytes;
use flate2::write::{GzDecoder, GzEncoder};
use pingora::protocols::http::compression::COMPRESSION_ERROR;
use pingora::{OrErr, Result};
use std::io::Write;
use std::time::{Duration, Instant};

pub trait Encode {
    /// Encode the input bytes. The `end` flag signals the end of the entire input. The `end` flag
    /// helps the encoder to flush out the remaining buffered encoded data because certain compression
    /// algorithms prefer to collect large enough data to compress all together.
    fn encode(&mut self, input: &[u8], end: bool) -> Result<Bytes>;
    /// Return the Encoder's name, the total input bytes, the total output bytes and the total
    /// duration spent on encoding the data.
    fn stat(&self) -> (&'static str, usize, usize, Duration);
}

pub struct Decompressor {
    decompress: GzDecoder<Vec<u8>>,
    total_in: usize,
    total_out: usize,
    duration: Duration,
}

impl Decompressor {
    pub fn new() -> Self {
        Decompressor {
            decompress: GzDecoder::new(vec![]),
            total_in: 0,
            total_out: 0,
            duration: Duration::new(0, 0),
        }
    }
}

impl Encode for Decompressor {
    fn encode(&mut self, input: &[u8], end: bool) -> Result<Bytes> {
        const MAX_INIT_COMPRESSED_SIZE_CAP: usize = 4 * 1024;
        const ESTIMATED_COMPRESSION_RATIO: usize = 3; // estimated 2.5-3x compression
        let start = Instant::now();
        self.total_in += input.len();
        // cap the buf size amplification, there is a DoS risk of always allocate
        // 3x the memory of the input buffer
        let reserve_size = if input.len() < MAX_INIT_COMPRESSED_SIZE_CAP {
            input.len() * ESTIMATED_COMPRESSION_RATIO
        } else {
            input.len()
        };
        self.decompress.get_mut().reserve(reserve_size);
        self.decompress
            .write_all(input)
            .or_err(COMPRESSION_ERROR, "while decompress Gzip")?;
        // write to vec will never fail, only possible error is that the input data
        // was not actually gzip compressed
        if end {
            self.decompress
                .try_finish()
                .or_err(COMPRESSION_ERROR, "while decompress Gzip")?;
        }
        self.total_out += self.decompress.get_ref().len();
        self.duration += start.elapsed();
        Ok(std::mem::take(self.decompress.get_mut()).into()) // into() Bytes will drop excess capacity
    }

    fn stat(&self) -> (&'static str, usize, usize, Duration) {
        ("de-gzip", self.total_in, self.total_out, self.duration)
    }
}

pub struct Compressor {
    // TODO: enum for other compression algorithms
    compress: GzEncoder<Vec<u8>>,
    total_in: usize,
    total_out: usize,
    duration: Duration,
}

impl Compressor {
    pub fn new(level: u32) -> Compressor {
        Compressor {
            compress: GzEncoder::new(vec![], flate2::Compression::new(level)),
            total_in: 0,
            total_out: 0,
            duration: Duration::new(0, 0),
        }
    }
}

impl Encode for Compressor {
    // infallible because compression can take any data
    fn encode(&mut self, input: &[u8], end: bool) -> Result<Bytes> {
        // reserve at most 16k
        const MAX_INIT_COMPRESSED_BUF_SIZE: usize = 16 * 1024;
        let start = Instant::now();
        self.total_in += input.len();
        self.compress
            .get_mut()
            .reserve(std::cmp::min(MAX_INIT_COMPRESSED_BUF_SIZE, input.len()));
        self.write_all(input).unwrap(); // write to vec, should never fail
        if end {
            self.try_finish().unwrap(); // write to vec, should never fail
        }
        self.total_out += self.compress.get_ref().len();
        self.duration += start.elapsed();
        Ok(std::mem::take(self.compress.get_mut()).into()) // into() Bytes will drop excess capacity
    }

    fn stat(&self) -> (&'static str, usize, usize, Duration) {
        ("gzip", self.total_in, self.total_out, self.duration)
    }
}

use std::ops::{Deref, DerefMut};
impl Deref for Decompressor {
    type Target = GzDecoder<Vec<u8>>;

    fn deref(&self) -> &Self::Target {
        &self.decompress
    }
}

impl DerefMut for Decompressor {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.decompress
    }
}

impl Deref for Compressor {
    type Target = GzEncoder<Vec<u8>>;

    fn deref(&self) -> &Self::Target {
        &self.compress
    }
}

impl DerefMut for Compressor {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.compress
    }
}

// ====================== ZSTD Compressor ======================

pub struct ZstdCompressor {
    compress: zstd::stream::write::Encoder<'static, Vec<u8>>,
    total_in: usize,
    total_out: usize,
    duration: Duration,
}

impl ZstdCompressor {
    pub fn new(level: i32) -> Self {
        let buf = Vec::new();
        let encoder = zstd::stream::write::Encoder::new(buf, level).unwrap();
        Self {
            compress: encoder,
            total_in: 0,
            total_out: 0,
            duration: Duration::new(0, 0),
        }
    }
}

impl Encode for ZstdCompressor {
    fn encode(&mut self, input: &[u8], end: bool) -> Result<Bytes> {
        let start = Instant::now();
        self.total_in += input.len();
        self.compress
            .get_mut()
            .reserve(std::cmp::min(16 * 1024, input.len()));
        self.compress.write_all(input).unwrap();
        if end {
            self.compress
                .do_finish()
                .unwrap();
        }
        self.total_out += self.compress.get_ref().len();
        self.duration += start.elapsed();
        Ok(std::mem::take(self.compress.get_mut()).into())
    }

    fn stat(&self) -> (&'static str, usize, usize, Duration) {
        ("zstd", self.total_in, self.total_out, self.duration)
    }
}

// ====================== ZSTD Decompressor ======================

pub struct ZstdDecompressor {
    decompress: zstd::stream::write::Decoder<'static, Vec<u8>>,
    total_in: usize,
    total_out: usize,
    duration: Duration,
}

impl ZstdDecompressor {
    pub fn new() -> Self {
        // Vec<u8> 作为输出缓冲
        let buf = Vec::new();
        let decoder = zstd::stream::write::Decoder::new(buf).unwrap();
        Self {
            decompress: decoder,
            total_in: 0,
            total_out: 0,
            duration: Duration::new(0, 0),
        }
    }
}

impl Encode for ZstdDecompressor {
    fn encode(&mut self, input: &[u8], end: bool) -> Result<Bytes> {
        let start = Instant::now();
        self.total_in += input.len();
        self.decompress.get_mut().reserve(input.len() * 2);
        self.decompress
            .write_all(input)
            .or_err(COMPRESSION_ERROR, "while decompress Zstd")?;
        if end {
            self.decompress
                .flush().unwrap();
        }
        self.total_out += self.decompress.get_ref().len();
        self.duration += start.elapsed();
        Ok(std::mem::take(self.decompress.get_mut()).into())
    }

    fn stat(&self) -> (&'static str, usize, usize, Duration) {
        ("de-zstd", self.total_in, self.total_out, self.duration)
    }
}

#[cfg(test)]
mod tests_stream {
    use super::*;

    #[test]
    fn gzip_data() {
        let mut compressor = Compressor::new(6);
        let compressed = compressor.encode(b"abcdefg", true).unwrap();
        // gzip magic headers
        assert_eq!(&compressed[..3], &[0x1f, 0x8b, 0x08]);
        // check the crc32 footer
        assert_eq!(
            &compressed[compressed.len() - 9..],
            &[0, 166, 106, 42, 49, 7, 0, 0, 0]
        );
        assert_eq!(compressor.total_in, 7);
        assert_eq!(compressor.total_out, compressed.len());

        assert!(compressor.get_ref().is_empty());
    }

    #[test]
    fn gunzip_data() {
        let mut decompressor = Decompressor::new();

        let compressed_bytes = &[
            0x1f, 0x8b, 0x08, 0, 0, 0, 0, 0, 0, 255, 75, 76, 74, 78, 73, 77, 75, 7, 0, 166, 106,
            42, 49, 7, 0, 0, 0,
        ];
        let decompressed = decompressor.encode(compressed_bytes, true).unwrap();

        assert_eq!(&decompressed[..], b"abcdefg");
        assert_eq!(decompressor.total_in, compressed_bytes.len());
        assert_eq!(decompressor.total_out, decompressed.len());

        assert!(decompressor.get_ref().is_empty());
    }
}
