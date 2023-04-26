//! Core local interruptor (timer interrupts).
use crate::hw::param;
use crate::hw::riscv;

/// Get the current CLINT time.
pub fn read_mtime() -> usize {
    let base = param::CLINT_BASE as *mut usize;
    let mtime: usize;
    unsafe {
        mtime = base.byte_add(0xBFF8).read_volatile();
    }
    mtime
}

/// Set the CLINT MTIMECMP register.
/// When CLINT MTIME >= CLINT MTIMECMP it triggers
/// a *machine*-mode interrupt.
// mtimecmp reg is at base + 0x4000
// mtime reg is base + 0xbff8
pub fn set_mtimecmp(interval: usize) {
    let hartid = riscv::get_mhartid();
    let base = param::CLINT_BASE as *mut usize;
    unsafe {
        // One mtime register for all cores.
        let mtime = base.byte_add(0xBFF8).read_volatile();
        // mtimecmp register per core.
        base.byte_add(0x4000 + 8 * hartid)
            .write_volatile(mtime + interval);
    }
}
