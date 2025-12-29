use axvfs::{VfsError, VfsResult};

pub type BlockId = u64;

pub trait BlockDevice {
    fn block_size(&self) -> usize;
    fn read_block(&self, block_id: BlockId, buf: &mut [u8]) -> VfsResult<()>;
    fn write_block(&self, block_id: BlockId, buf: &[u8]) -> VfsResult<()>;
    fn flush(&self) -> VfsResult<()>;
}

pub struct BlockCache<'a> {
    device: &'a dyn BlockDevice,
    block_size: usize,
}

impl<'a> BlockCache<'a> {
    pub fn new(device: &'a dyn BlockDevice) -> Self {
        let block_size = device.block_size();
        Self { device, block_size }
    }

    pub fn block_size(&self) -> usize {
        self.block_size
    }

    pub fn read_block(&self, block_id: BlockId, buf: &mut [u8]) -> VfsResult<()> {
        if buf.len() < self.block_size {
            return Err(VfsError::Invalid);
        }
        self.device.read_block(block_id, &mut buf[..self.block_size])
    }

    pub fn write_block(&self, block_id: BlockId, buf: &[u8]) -> VfsResult<()> {
        if buf.len() < self.block_size {
            return Err(VfsError::Invalid);
        }
        self.device.write_block(block_id, &buf[..self.block_size])
    }

    pub fn flush(&self) -> VfsResult<()> {
        self.device.flush()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use core::cell::RefCell;

    struct TestBlockDevice {
        block_size: usize,
        data: RefCell<[u8; 8]>,
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

        fn write_block(&self, block_id: BlockId, buf: &[u8]) -> VfsResult<()> {
            let offset = block_id as usize * self.block_size;
            let mut data = self.data.borrow_mut();
            if offset + self.block_size > data.len() {
                return Err(VfsError::NotFound);
            }
            data[offset..offset + self.block_size].copy_from_slice(&buf[..self.block_size]);
            Ok(())
        }

        fn flush(&self) -> VfsResult<()> {
            Ok(())
        }
    }

    #[test]
    fn block_cache_passthrough() {
        let dev = TestBlockDevice {
            block_size: 4,
            data: RefCell::new([0u8; 8]),
        };
        let cache = BlockCache::new(&dev);
        let buf = [1u8, 2, 3, 4];
        cache.write_block(1, &buf).unwrap();
        let mut read = [0u8; 4];
        cache.read_block(1, &mut read).unwrap();
        assert_eq!(read, buf);
    }
}
