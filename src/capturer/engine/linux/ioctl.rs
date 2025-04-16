// https://github.com/mripard/dma-buf/blob/main/src/ioctl.rs

use std::os::fd::BorrowedFd;

use rustix::{
    io::Errno,
    ioctl::{ioctl, opcode, Setter},
};

use super::error::LinCapError;

const DMA_BUF_BASE: u8 = b'b';
const DMA_BUF_IOCTL_SYNC: u8 = 0;

const DMA_BUF_SYNC_READ: u64 = 1 << 0;
const DMA_BUF_SYNC_START: u64 = 0 << 2;
const DMA_BUF_SYNC_END: u64 = 1 << 2;

#[derive(Default)]
#[repr(C)]
struct DmaBufSync {
    flags: u64,
}

fn dma_buf_sync_ioctl(fd: BorrowedFd<'_>, flags: u64) -> Result<(), Errno> {
    let sync = DmaBufSync { flags };

    // SAFETY: This function is unsafe because the opcode has to be valid, and the value type must
    // match. We have checked those, so we're good.
    let ioctl_type = unsafe {
        Setter::<{ opcode::write::<DmaBufSync>(DMA_BUF_BASE, DMA_BUF_IOCTL_SYNC) }, DmaBufSync>::new(
            sync,
        )
    };

    // SAFETY: This function is unsafe because the driver isn't guaranteed to implement the ioctl,
    // and to implement it properly. We don't have much of a choice and still have to trust the
    // kernel there.
    unsafe { ioctl(fd, ioctl_type) }
}

fn dma_buf_sync(fd: BorrowedFd<'_>, flags: u64) -> Result<(), LinCapError> {
    dma_buf_sync_ioctl(fd, flags)?;
    Ok(())
}

pub(crate) fn dma_buf_begin_cpu_read_access(fd: BorrowedFd<'_>) -> Result<(), LinCapError> {
    dma_buf_sync(fd, DMA_BUF_SYNC_START | DMA_BUF_SYNC_READ)?;
    Ok(())
}

pub(crate) fn dma_buf_end_cpu_read_access(fd: BorrowedFd<'_>) -> Result<(), LinCapError> {
    dma_buf_sync(fd, DMA_BUF_SYNC_END | DMA_BUF_SYNC_READ)?;
    Ok(())
}
