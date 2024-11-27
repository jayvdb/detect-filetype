#[non_exhaustive]
#[derive(Copy, Clone, PartialEq, Eq, Debug, Hash)]
pub enum FileType {
    // -- Images --
    Tga,
    Jpeg,
    Png,
    Bmp,
    Tiff,
    BigTiff,

    // -- Compression/Archives --
    Zip,
    Bzip2,
    /// Tar archive format.
    ///
    /// For some reason empty tar files don't have this number, only tar files with at least
    /// one element
    Tar,
}

impl FileType {
    pub fn extension(&self) -> &'static str {
        match self {
            FileType::Tga => "tga",
            FileType::Jpeg => "jpg",
            FileType::Png => "png",
            FileType::Bmp => "bmp",
            FileType::Tiff => "tif",
            FileType::BigTiff => "tif",
            FileType::Zip => "zip",
            FileType::Bzip2 => "bz2",
            FileType::Tar => "tar",
        }
    }
}

struct Check {
    bytes: &'static [u8],
    offset: usize,
}

impl Check {
    const fn default() -> Self {
        const EMPTY: &[u8] = &[];

        Check {
            bytes: EMPTY,
            offset: 0,
        }
    }

    const fn new(bytes: &'static [u8]) -> Self {
        Self::new_with_offset(0, bytes)
    }

    const fn new_with_offset(offset: usize, bytes: &'static [u8]) -> Self {
        Check { bytes, offset }
    }
}

impl Default for Check {
    fn default() -> Self {
        Check::default()
    }
}

struct Magic {
    start: Check,
    end: Check,
}

impl Magic {
    const fn starts_with(bytes: &'static [u8]) -> Self {
        Magic {
            start: Check::new(bytes),
            end: Check::default(),
        }
    }

    const fn starts_with_offset(offset: usize, bytes: &'static [u8]) -> Self {
        Magic {
            start: Check::new_with_offset(offset, bytes),
            end: Check::default(),
        }
    }

    const fn ends_with(bytes: &'static [u8]) -> Self {
        Magic {
            start: Check::default(),
            end: Check { bytes, offset: 0 },
        }
    }
}

const MAGIC_MAP: &[(Magic, FileType)] = &[
    (Magic::ends_with(b"TRUEVISION-XFILE.\0"), FileType::Tga),
    (Magic::starts_with(&[0xff, 0xd8]), FileType::Jpeg),
    (
        Magic {
            start: Check::new(&[0x89, 0x50, 0x4e, 0x47, 0x0d, 0x0a, 0x1a, 0x0a]),
            end: Check::new(&[0x49, 0x45, 0x4e, 0x44, 0xae, 0x42, 0x60, 0x82]),
        },
        FileType::Png,
    ),
    (Magic::starts_with(b"BM"), FileType::Bmp),
    (Magic::starts_with(b"II\x2a\x00"), FileType::Tiff), // Little Endian
    (Magic::starts_with(b"MM\x00\x2a"), FileType::Tiff), // Big Endian
    (Magic::starts_with(b"II\x2b\x00"), FileType::BigTiff), // Little Endian
    (Magic::starts_with(b"MM\x00\x2b"), FileType::BigTiff), // Big Endian
    (Magic::starts_with(b"BZh"), FileType::Bzip2),
    (Magic::starts_with(&[0x50, 0x4b, 0x03, 0x04]), FileType::Zip),
    (Magic::starts_with(&[0x50, 0x4b, 0x05, 0x06]), FileType::Zip),
    (Magic::starts_with_offset(0x1e, b"PKLITE"), FileType::Zip),
    (
        Magic::starts_with_offset(0x101, b"ustar  \0"),
        FileType::Tar,
    ),
];

pub fn detect_filetype(bytes: &[u8]) -> Option<FileType> {
    for (magic, ty) in MAGIC_MAP {
        if bytes[magic.start.offset..].starts_with(magic.start.bytes)
            && bytes[..bytes.len() - magic.end.offset].ends_with(magic.end.bytes)
        {
            return Some(*ty);
        }
    }

    None
}

#[cfg(test)]
mod tests {
    use super::{detect_filetype, FileType};
    use std::{
        fs,
        io::{self, Read},
        path::Path,
    };

    fn get_bytes(path: impl AsRef<Path>) -> io::Result<Vec<u8>> {
        let mut file = fs::File::open(path)?;
        let mut out = Vec::new();
        file.read_to_end(&mut out)?;
        Ok(out)
    }

    macro_rules! file_test {
        ($extension:ident, $variant:ident) => {
            #[test]
            fn $extension() -> io::Result<()> {
                assert_eq!(
                    detect_filetype(&get_bytes(concat!("test.", stringify!($extension)))?),
                    Some(FileType::$variant)
                );

                Ok(())
            }
        };
    }

    file_test!(tga, Tga);
    file_test!(jpg, Jpeg);
    file_test!(png, Png);
    file_test!(bmp, Bmp);
    file_test!(tif, Tiff);
    file_test!(bif, BigTiff);
    file_test!(zip, Zip);
    file_test!(bz2, Bzip2);
    file_test!(tar, Tar);
}
