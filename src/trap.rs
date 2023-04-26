//! Kernel trap handlers.
use crate::device::clint;
use crate::hw::riscv::{get_mcause, get_mhartid, get_scause};
use crate::hw::{riscv, INTERVAL};
use crate::vm::ptable::PageTable;

use crate::log;

// TODO currently we are using machine mode direct for everything. So
// everything goes though mhandler. The situation has become more
// complicated, see the issue on github about it.

extern "C" {
    pub fn __mtrapvec();
    pub fn __strapvec();
}

pub struct TrapFrame {
    kpgtbl: *mut PageTable,
    handler: *const (),
    cause: usize,
    retpc: usize, // Return from trap program counter value.
    regs: [usize; 32],
}

/// Write the supervisor trap vector to stvec register on each hart.
pub fn init() {
    riscv::set_stvec(__strapvec as usize);
}

/// Machine mode trap handler.
#[no_mangle]
pub extern "C" fn m_handler() {
    let cause = get_mcause();
    let is_interrupt = (get_mcause() as isize) < 0;

    match cause {
        // Machine timer interrupt
        7 if is_interrupt => {
            log::log!(Debug, "Machine timer interrupt, hart: {}", get_mhartid());
            clint::set_mtimecmp(10_000_000);
        }
        _ => {
            log::log!(
                Warning,
                "Uncaught machine mode interrupt. mcause: 0x{:x}",
                cause
            );
            panic!();
        }
    }
}

/// Supervisor mode trap handler.
#[no_mangle]
pub extern "C" fn s_handler() {
    let cause = get_scause();

    {
        log::log!(
            Warning,
            "Uncaught supervisor mode interupt. scause: 0x{:x}",
            cause
        );
        panic!()
    }
}
