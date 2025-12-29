use axvfs::{FileType, InodeId, Metadata, VfsError, VfsOps, VfsResult};

use crate::block::{BlockCache, BlockDevice, BlockId};

const EXT4_ROOT_INODE: InodeId = 2;
const EXT4_MAGIC: u16 = 0xef53;
const SUPERBLOCK_OFFSET: u64 = 1024;
const SUPERBLOCK_SIZE: usize = 1024;
const SUPERBLOCK_LOG_BLOCK_SIZE_OFFSET: usize = 24;
const SUPERBLOCK_BLOCKS_PER_GROUP_OFFSET: usize = 32;
const SUPERBLOCK_INODES_PER_GROUP_OFFSET: usize = 40;
const SUPERBLOCK_MAGIC_OFFSET: usize = 56;
const SUPERBLOCK_INODE_SIZE_OFFSET: usize = 88;

#[derive(Clone, Copy, Debug)]
pub struct SuperBlock {
    pub log_block_size: u32,
    pub blocks_per_group: u32,
    pub inodes_per_group: u32,
    pub inode_size: u16,
    pub magic: u16,
}

impl SuperBlock {
    pub fn parse(buf: &[u8]) -> VfsResult<Self> {
        if buf.len() < SUPERBLOCK_SIZE {
            return Err(VfsError::Invalid);
        }
        let magic = read_u16(buf, SUPERBLOCK_MAGIC_OFFSET);
        if magic != EXT4_MAGIC {
            return Err(VfsError::Invalid);
        }
        let log_block_size = read_u32(buf, SUPERBLOCK_LOG_BLOCK_SIZE_OFFSET);
        let blocks_per_group = read_u32(buf, SUPERBLOCK_BLOCKS_PER_GROUP_OFFSET);
        let inodes_per_group = read_u32(buf, SUPERBLOCK_INODES_PER_GROUP_OFFSET);
        let inode_size = read_u16(buf, SUPERBLOCK_INODE_SIZE_OFFSET);
        let block_size = 1024u32.checked_shl(log_block_size).ok_or(VfsError::Invalid)?;
        if block_size < 1024 || !block_size.is_power_of_two() || inode_size == 0 {
            return Err(VfsError::Invalid);
        }
        Ok(Self {
            log_block_size,
            blocks_per_group,
            inodes_per_group,
            inode_size,
            magic,
        })
    }

    pub fn block_size(&self) -> u32 {
        1024u32 << self.log_block_size
    }
}

pub struct Ext4Fs<'a> {
    cache: BlockCache<'a>,
    superblock: SuperBlock,
}

impl<'a> Ext4Fs<'a> {
    pub fn new(device: &'a dyn BlockDevice) -> VfsResult<Self> {
        let cache = BlockCache::new(device);
        let block_size = cache.block_size();
        if block_size == 0 || block_size > 4096 {
            return Err(VfsError::Invalid);
        }
        let mut buf = [0u8; SUPERBLOCK_SIZE];
        read_bytes(&cache, SUPERBLOCK_OFFSET, &mut buf)?;
        let superblock = SuperBlock::parse(&buf)?;
        Ok(Self { cache, superblock })
    }

    pub fn superblock(&self) -> &SuperBlock {
        &self.superblock
    }

    pub fn fs_block_size(&self) -> u32 {
        self.superblock.block_size()
    }

    pub fn read_block(&self, block: BlockId, buf: &mut [u8]) -> VfsResult<()> {
        self.cache.read_block(block, buf)
    }
}

impl VfsOps for Ext4Fs<'_> {
    fn root(&self) -> VfsResult<InodeId> {
        Ok(EXT4_ROOT_INODE)
    }

    fn lookup(&self, _parent: InodeId, _name: &str) -> VfsResult<Option<InodeId>> {
        Err(VfsError::NotSupported)
    }

    fn create(&self, _parent: InodeId, _name: &str, _kind: FileType, _mode: u16) -> VfsResult<InodeId> {
        Err(VfsError::NotSupported)
    }

    fn remove(&self, _parent: InodeId, _name: &str) -> VfsResult<()> {
        Err(VfsError::NotSupported)
    }

    fn metadata(&self, inode: InodeId) -> VfsResult<Metadata> {
        if inode == EXT4_ROOT_INODE {
            Ok(Metadata::new(FileType::Dir, 0, 0o755))
        } else {
            Err(VfsError::NotSupported)
        }
    }

    fn read_at(&self, _inode: InodeId, _offset: u64, _buf: &mut [u8]) -> VfsResult<usize> {
        Err(VfsError::NotSupported)
    }

    fn write_at(&self, _inode: InodeId, _offset: u64, _buf: &[u8]) -> VfsResult<usize> {
        Err(VfsError::NotSupported)
    }
}

fn read_bytes(cache: &BlockCache<'_>, offset: u64, buf: &mut [u8]) -> VfsResult<()> {
    let block_size = cache.block_size();
    if block_size == 0 || block_size > 4096 {
        return Err(VfsError::Invalid);
    }
    let block_size_u64 = block_size as u64;
    let mut scratch = [0u8; 4096];
    let mut remaining = buf.len();
    let mut buf_offset = 0usize;
    let mut cur_offset = offset;
    while remaining > 0 {
        let block_id = cur_offset / block_size_u64;
        let in_block = (cur_offset % block_size_u64) as usize;
        let to_copy = core::cmp::min(remaining, block_size - in_block);
        cache.read_block(block_id, &mut scratch[..block_size])?;
        buf[buf_offset..buf_offset + to_copy]
            .copy_from_slice(&scratch[in_block..in_block + to_copy]);
        remaining -= to_copy;
        buf_offset += to_copy;
        cur_offset += to_copy as u64;
    }
    Ok(())
}

fn read_u16(buf: &[u8], offset: usize) -> u16 {
    u16::from_le_bytes([buf[offset], buf[offset + 1]])
}

fn read_u32(buf: &[u8], offset: usize) -> u32 {
    u32::from_le_bytes([buf[offset], buf[offset + 1], buf[offset + 2], buf[offset + 3]])
}

#[cfg(test)]
mod tests {
    use super::*;
    use core::cell::RefCell;

    struct TestBlockDevice {
        block_size: usize,
        data: RefCell<[u8; 2048]>,
    }

    impl BlockDevice for TestBlockDevice {
        fn block_size(&self) -> usize {
            self.block_size
        }

        fn read_block(&self, block_id: BlockId, buf: &mut [u8]) -> VfsResult<()> {
            let offset = block_id as usize * self.block_size;
            let data = self.data.borrow();
            if offset + self.block_size > data.len() {
                return Err(VfsError::NotFound);
            }
            buf[..self.block_size].copy_from_slice(&data[offset..offset + self.block_size]);
            Ok(())
        }

        fn write_block(&self, _block_id: BlockId, _buf: &[u8]) -> VfsResult<()> {
            Err(VfsError::NotSupported)
        }

        fn flush(&self) -> VfsResult<()> {
            Ok(())
        }
    }

    #[test]
    fn parse_superblock() {
        let mut data = [0u8; 2048];
        let sb = &mut data[SUPERBLOCK_OFFSET as usize..SUPERBLOCK_OFFSET as usize + SUPERBLOCK_SIZE];
        sb[SUPERBLOCK_MAGIC_OFFSET..SUPERBLOCK_MAGIC_OFFSET + 2].copy_from_slice(&EXT4_MAGIC.to_le_bytes());
        sb[SUPERBLOCK_LOG_BLOCK_SIZE_OFFSET..SUPERBLOCK_LOG_BLOCK_SIZE_OFFSET + 4]
            .copy_from_slice(&0u32.to_le_bytes());
        sb[SUPERBLOCK_BLOCKS_PER_GROUP_OFFSET..SUPERBLOCK_BLOCKS_PER_GROUP_OFFSET + 4]
            .copy_from_slice(&8192u32.to_le_bytes());
        sb[SUPERBLOCK_INODES_PER_GROUP_OFFSET..SUPERBLOCK_INODES_PER_GROUP_OFFSET + 4]
            .copy_from_slice(&2048u32.to_le_bytes());
        sb[SUPERBLOCK_INODE_SIZE_OFFSET..SUPERBLOCK_INODE_SIZE_OFFSET + 2]
            .copy_from_slice(&128u16.to_le_bytes());
        let dev = TestBlockDevice {
            block_size: 512,
            data: RefCell::new(data),
        };
        let fs = Ext4Fs::new(&dev).unwrap();
        assert_eq!(fs.superblock().magic, EXT4_MAGIC);
        assert_eq!(fs.fs_block_size(), 1024);
    }
}
