use core::{alloc::Layout, arch::asm, ffi::CStr, time::Duration};

use axhal::mem::{phys_to_virt, virt_to_phys};
use axmm::PhysAddr;
use axtask::sleep;
use log::debug;
extern crate alloc;

/*
// for C ffi test
unsafe extern "C" {
    pub fn ahci_mdelay(ms: u32);
    pub fn ahci_printf(fmt: *const u8, _: ...) -> i32;
    pub fn ahci_malloc_align(size: u64, align: u32) -> u64;
    pub fn ahci_sync_dcache();
    pub fn ahci_phys_to_uncached(va: u64) -> u64;
    pub fn ahci_virt_to_phys(va: u64) -> u64;
}
*/

// 这里是测试时用于调用C的printf
// 替换成OS实现的printf
/*
 *unsafe extern "C" {
 *    pub fn ahci_printf(fmt: *const u8, _: ...) -> i32;
 *}
 */


#[macro_export]
macro_rules! ahci_printf {
    ($($arg:tt)*) => ({
        let formatted_string = format!($($arg)*);
        debug!("ahci: {}", formatted_string);
        1
    });
}

/*
 *pub fn ahci_printf(fmt: *const u8) -> i32 {
 *    let s = unsafe { CStr::from_ptr(fmt as *const i8) };
 *    debug!("achi: {}", str);
 *    1
 *}
 */

// 等待数毫秒
pub fn ahci_mdelay(ms: u64) {
    sleep(Duration::from_millis(ms));
}

// 同步dcache中所有cached和uncached访存请求
pub fn ahci_sync_dcache() {
    unsafe {
        asm!("dbar 0");
    }
}


/// 分配指定大小的内存，并确保内存地址按指定对齐。
///
/// 由于该操作用于硬件初始化，因此允许内存泄漏。
///
/// # Arguments
/// * `size` - 要分配的内存字节数。
/// * `align` - 内存的对齐要求（必须是2的幂）。
///
/// # Returns
/// 分配内存的物理地址（实际上是虚拟地址，但对于OS内核或裸机环境，
/// 通常直接映射到物理地址或可以直接当作物理地址处理）。
/// 如果分配失败，将返回0。
///
/// # Safety
/// 这个函数是不安全的，因为它直接调用了底层的 `alloc` API，
/// 并且在分配失败时返回 `0` 而不是通过 `panic` 或 `Result` 汇报。
/// 调用者需要确保对齐参数有效（2的幂），并且需要处理返回 `0` 的情况。
pub fn ahci_malloc_align(size: u64, align: u32) -> u64 {
    if !align.is_power_of_two() {
        return 0; 
    }

    let layout = unsafe {
        Layout::from_size_align(size as usize, align as usize)
            .unwrap_or_else(|_| {
                panic!("Invalid memory layout for AHCI allocation: size={}, align={}", size, align);
            })
    };

    let ptr = unsafe { alloc::alloc::alloc(layout) };

    if ptr.is_null() {
        0
    } else {
        ptr as u64
    }
}

// 物理地址转换为uncached虚拟地址
pub fn ahci_phys_to_uncached(pa: u64) -> u64 {
    phys_to_virt((pa as usize).into()).as_usize() as u64
}

// cached虚拟地址转换为物理地址
// ahci dma可以接受64位的物理地址
pub fn ahci_virt_to_phys(va: u64) -> u64 {
    virt_to_phys((va as usize).into()).as_usize() as u64
}
