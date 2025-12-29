use axvfs::{FileType, InodeId, Metadata, VfsError, VfsOps, VfsResult};

use crate::block::{BlockCache, BlockDevice, BlockId};

const BPB_SIZE: usize = 512;
const BPB_SIGNATURE_OFFSET: usize = 510;
const BPB_SIGNATURE: u16 = 0xaa55;
const BPB_BYTES_PER_SECTOR_OFFSET: usize = 11;
const BPB_SECTORS_PER_CLUSTER_OFFSET: usize = 13;
const BPB_RESERVED_SECTORS_OFFSET: usize = 14;
const BPB_NUM_FATS_OFFSET: usize = 16;
const BPB_ROOT_ENTRIES_OFFSET: usize = 17;
const BPB_TOTAL_SECTORS_16_OFFSET: usize = 19;
const BPB_FAT_SIZE_16_OFFSET: usize = 22;
const BPB_TOTAL_SECTORS_32_OFFSET: usize = 32;
const BPB_FAT_SIZE_32_OFFSET: usize = 36;
const BPB_ROOT_CLUSTER_OFFSET: usize = 44;

#[derive(Clone, Copy, Debug)]
pub struct Bpb {
    pub bytes_per_sector: u16,
    pub sectors_per_cluster: u8,
    pub reserved_sectors: u16,
    pub num_fats: u8,
    pub total_sectors: u32,
    pub sectors_per_fat: u32,
    pub root_cluster: u32,
}

impl Bpb {
    pub fn parse(buf: &[u8]) -> VfsResult<Self> {
        if buf.len() < BPB_SIZE {
            return Err(VfsError::Invalid);
        }
        let sig = u16::from_le_bytes([buf[BPB_SIGNATURE_OFFSET], buf[BPB_SIGNATURE_OFFSET + 1]]);
        if sig != BPB_SIGNATURE {
            return Err(VfsError::Invalid);
        }
        let bytes_per_sector = u16::from_le_bytes([
            buf[BPB_BYTES_PER_SECTOR_OFFSET],
            buf[BPB_BYTES_PER_SECTOR_OFFSET + 1],
        ]);
        let sectors_per_cluster = buf[BPB_SECTORS_PER_CLUSTER_OFFSET];
        let reserved_sectors = u16::from_le_bytes([
            buf[BPB_RESERVED_SECTORS_OFFSET],
            buf[BPB_RESERVED_SECTORS_OFFSET + 1],
        ]);
        let num_fats = buf[BPB_NUM_FATS_OFFSET];
        let root_entries = u16::from_le_bytes([
            buf[BPB_ROOT_ENTRIES_OFFSET],
            buf[BPB_ROOT_ENTRIES_OFFSET + 1],
        ]);
        let total_sectors_16 = u16::from_le_bytes([
            buf[BPB_TOTAL_SECTORS_16_OFFSET],
            buf[BPB_TOTAL_SECTORS_16_OFFSET + 1],
        ]);
        let fat_size_16 = u16::from_le_bytes([
            buf[BPB_FAT_SIZE_16_OFFSET],
            buf[BPB_FAT_SIZE_16_OFFSET + 1],
        ]);
        let total_sectors_32 = u32::from_le_bytes([
            buf[BPB_TOTAL_SECTORS_32_OFFSET],
            buf[BPB_TOTAL_SECTORS_32_OFFSET + 1],
            buf[BPB_TOTAL_SECTORS_32_OFFSET + 2],
            buf[BPB_TOTAL_SECTORS_32_OFFSET + 3],
        ]);
        let fat_size_32 = u32::from_le_bytes([
            buf[BPB_FAT_SIZE_32_OFFSET],
            buf[BPB_FAT_SIZE_32_OFFSET + 1],
            buf[BPB_FAT_SIZE_32_OFFSET + 2],
            buf[BPB_FAT_SIZE_32_OFFSET + 3],
        ]);
        let root_cluster = u32::from_le_bytes([
            buf[BPB_ROOT_CLUSTER_OFFSET],
            buf[BPB_ROOT_CLUSTER_OFFSET + 1],
            buf[BPB_ROOT_CLUSTER_OFFSET + 2],
            buf[BPB_ROOT_CLUSTER_OFFSET + 3],
        ]);
        let total_sectors = if total_sectors_16 != 0 {
            total_sectors_16 as u32
        } else {
            total_sectors_32
        };
        let sectors_per_fat = if fat_size_16 != 0 {
            fat_size_16 as u32
        } else {
            fat_size_32
        };
        if bytes_per_sector == 0
            || sectors_per_cluster == 0
            || reserved_sectors == 0
            || num_fats == 0
            || total_sectors == 0
            || sectors_per_fat == 0
            || root_entries != 0
            || root_cluster < 2
        {
            return Err(VfsError::Invalid);
        }
        Ok(Self {
            bytes_per_sector,
            sectors_per_cluster,
            reserved_sectors,
            num_fats,
            total_sectors,
            sectors_per_fat,
            root_cluster,
        })
    }

    pub fn fat_start_sector(&self) -> u32 {
        self.reserved_sectors as u32
    }

    pub fn data_start_sector(&self) -> u32 {
        self.fat_start_sector() + self.sectors_per_fat * self.num_fats as u32
    }
}

pub struct Fat32Fs<'a> {
    cache: BlockCache<'a>,
    bpb: Bpb,
}

impl<'a> Fat32Fs<'a> {
    pub fn new(device: &'a dyn BlockDevice) -> VfsResult<Self> {
        let cache = BlockCache::new(device);
        let block_size = cache.block_size();
        if block_size < BPB_SIZE || block_size > 4096 {
            return Err(VfsError::Invalid);
        }
        let mut sector = [0u8; 4096];
        cache.read_block(0, &mut sector[..block_size])?;
        let bpb = Bpb::parse(&sector[..block_size])?;
        if bpb.bytes_per_sector as usize != block_size {
            return Err(VfsError::Invalid);
        }
        Ok(Self { cache, bpb })
    }

    pub fn bpb(&self) -> &Bpb {
        &self.bpb
    }

    pub fn cluster_to_sector(&self, cluster: u32) -> u32 {
        let cluster_index = cluster.saturating_sub(2);
        self.bpb.data_start_sector()
            + cluster_index * self.bpb.sectors_per_cluster as u32
    }

    pub fn read_sector(&self, sector: BlockId, buf: &mut [u8]) -> VfsResult<()> {
        self.cache.read_block(sector, buf)
    }
}

impl VfsOps for Fat32Fs<'_> {
    fn root(&self) -> VfsResult<InodeId> {
        Ok(self.bpb.root_cluster as InodeId)
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
        if inode == self.bpb.root_cluster as InodeId {
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

#[cfg(test)]
mod tests {
    use super::*;
    use core::cell::RefCell;

    struct TestBlockDevice {
        block_size: usize,
        data: RefCell<[u8; 512]>,
    }

    impl BlockDevice for TestBlockDevice {
        fn block_size(&self) -> usize {
            self.block_size
        }

        fn read_block(&self, block_id: BlockId, buf: &mut [u8]) -> VfsResult<()> {
            if block_id != 0 {
                return Err(VfsError::NotFound);
            }
            let data = self.data.borrow();
            buf[..self.block_size].copy_from_slice(&data[..self.block_size]);
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
    fn parse_bpb() {
        let mut data = [0u8; 512];
        data[BPB_SIGNATURE_OFFSET] = 0x55;
        data[BPB_SIGNATURE_OFFSET + 1] = 0xaa;
        data[BPB_BYTES_PER_SECTOR_OFFSET..BPB_BYTES_PER_SECTOR_OFFSET + 2]
            .copy_from_slice(&512u16.to_le_bytes());
        data[BPB_SECTORS_PER_CLUSTER_OFFSET] = 1;
        data[BPB_RESERVED_SECTORS_OFFSET..BPB_RESERVED_SECTORS_OFFSET + 2]
            .copy_from_slice(&32u16.to_le_bytes());
        data[BPB_NUM_FATS_OFFSET] = 2;
        data[BPB_TOTAL_SECTORS_32_OFFSET..BPB_TOTAL_SECTORS_32_OFFSET + 4]
            .copy_from_slice(&2048u32.to_le_bytes());
        data[BPB_FAT_SIZE_32_OFFSET..BPB_FAT_SIZE_32_OFFSET + 4]
            .copy_from_slice(&1u32.to_le_bytes());
        data[BPB_ROOT_CLUSTER_OFFSET..BPB_ROOT_CLUSTER_OFFSET + 4]
            .copy_from_slice(&2u32.to_le_bytes());
        let dev = TestBlockDevice {
            block_size: 512,
            data: RefCell::new(data),
        };
        let fs = Fat32Fs::new(&dev).unwrap();
        assert_eq!(fs.bpb().bytes_per_sector, 512);
        assert_eq!(fs.bpb().root_cluster, 2);
        assert_eq!(fs.cluster_to_sector(2), fs.bpb().data_start_sector());
    }
}
