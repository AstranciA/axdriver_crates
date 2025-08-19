use axdriver_base::{BaseDriverOps, DeviceType};
use log::warn;

use crate::{
    ahci::{
        drv_ahci::{
            ahci_init, ahci_sata_flush_cache_ext, ahci_sata_read_common, ahci_sata_write_common,
        },
        libahci::{ahci_blk_dev, ahci_device, ahci_ioport},
    },
    BlockDriverOps,
};

mod drv_ahci;
mod libahci;
mod libata;
mod platform;

pub struct AhciDriver(ahci_device);

impl AhciDriver {
    pub fn new() -> Self {
        Self(unsafe { core::mem::zeroed() })
    }
}

impl Default for AhciDriver {
    fn default() -> Self {
        Self::new()
    }
}

impl BaseDriverOps for AhciDriver {
    fn device_name(&self) -> &str {
        "ahci"
    }

    fn device_type(&self) -> DeviceType {
        DeviceType::Block
    }

    fn init(&mut self) {
        ahci_init(&mut self.0);
    }

    fn mmio_base(&self) -> usize {
        self.0.mmio_base as usize
    }

    fn mmio_size(&self) -> usize {
        0x10000
    }
}

impl BlockDriverOps for AhciDriver {
    fn num_blocks(&self) -> u64 {
        32 * 1024 * 1024 * 1024 / self.block_size() as u64
    }

    fn block_size(&self) -> usize {
        512
    }

    fn read_block(&mut self, block_id: u64, buf: &mut [u8]) -> axdriver_base::DevResult {
        assert!(buf.len() % self.block_size() == 0);
        let blkcnt = buf.len() / self.block_size();
        ahci_sata_read_common(
            &self.0,
            block_id,
            blkcnt.try_into().unwrap(),
            buf.as_mut_ptr(),
        );
        Ok(())
    }

    fn write_block(&mut self, block_id: u64, buf: &[u8]) -> axdriver_base::DevResult {
        assert!(buf.len() % self.block_size() == 0);
        let blkcnt = buf.len() / self.block_size();
        unsafe {
            ahci_sata_write_common(
                &self.0,
                block_id,
                blkcnt.try_into().unwrap(),
                buf.as_ptr() as usize as *mut u8,
            );
        }
        Ok(())
    }

    fn flush(&mut self) -> axdriver_base::DevResult {
        ahci_sata_flush_cache_ext(&self.0);
        Ok(())
    }
}
