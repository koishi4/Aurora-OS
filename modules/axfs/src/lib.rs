#![no_std]

pub mod memfs;

pub use axvfs::{FileType, InodeId, Metadata, VfsError, VfsOps};

#[cfg(test)]
extern crate std;
