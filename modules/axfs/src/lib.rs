#![no_std]

pub mod block;
pub mod devfs;
pub mod fat32;
pub mod memfs;
pub mod mount;
pub mod procfs;

pub use axvfs::{FileType, InodeId, Metadata, VfsError, VfsOps};

#[cfg(test)]
extern crate std;
