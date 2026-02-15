pub mod compress;
pub mod decompress;

pub mod prelude {
    pub use super::compress::compress;
    pub use super::decompress::{
        new_dict_decompressor,
        new_decompressor_wrapper,
        decompress_with_decompressor_wrapper,
        decompress_with_dict_decompressor,
        decompress,
        DecompressorWrapper,
    };
}

mod _internals {
    pub const ZSTD_MAGIC_NUM : &[u8;4] = b"\x28\xB5\x2F\xFD";

    pub const SIZE_TREE_LEN :usize = 4; // size of tree item length value in file : 32 bits = 4 byte
    pub const SIZE_TREE_SIZE :usize = 4; // size of tree size value in file : 32 bits = 4 byte
    pub const SIZE_HASH_VAL :usize = 4; // size of hash value in file : 32 bits = 4 byte
    pub const SIZE_ABS_POS :usize = 4; // size of absolute archive position value in file : 32 bits = 4 byte

    pub type Map<'a> = FxHashMap<&'a str, u32>;

    pub use std::{
        hash::BuildHasher,
        fs::{File, OpenOptions},
        io::{Read, Write, Seek, SeekFrom, },
        path::Path,
    };
    pub use rustc_hash::{FxHashMap, FxBuildHasher};
    pub use walkdir::WalkDir;
    pub use cfg_log::debug;
    pub use anyhow::{Result, anyhow};    
    pub use zstd_safe::{CCtx, DCtx};
}

