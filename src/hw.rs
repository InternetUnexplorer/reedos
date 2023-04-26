//! Target-hardware parameters and utilities.
pub mod param;
pub mod riscv;

use crate::device::clint;
use crate::process::Process;
use crate::trap;
use riscv::*;

pub const INTERVAL: usize = 10_000_000;

/// Callee saved registers.
pub struct HartContext {
    regs: [usize; 32],
}

/// Representation of riscv hart.
pub struct Hart {
    id: usize,
    process: Process,
    ctx_regs: HartContext,
}

/// Set up and enable the core local interrupt controller on each hart.
/// We write the machine mode trap vector register (mtvec) with the address
/// of our `src/asm` trap handler function.
pub fn timerinit() {
    clint::set_mtimecmp(INTERVAL);

    // Set the machine trap vector to hold fn ptr to timervec.
    set_mtvec(trap::__mtrapvec as usize);

    // Enable machine mode interrupts (mstatus MIE).
    set_mstatus(get_mstatus() | (1 << 3));
    // Enable machine-mode timer interrupts (mie MTIE).
    set_mie(get_mie() | (1 << 7));
}
