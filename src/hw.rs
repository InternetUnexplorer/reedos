//! Target-hardware parameters and utilities.
pub mod param;
pub mod riscv;

use crate::device::clint;
use crate::process::Process;
use crate::trap;
use riscv::*;

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
    let interval = 10_000_000; // May want to speed this up in the future.
    clint::set_mtimecmp(interval);

    // Set the machine trap vector to hold fn ptr to timervec.
    let timervec_fn = trap::__mtrapvec;
    set_mtvec(timervec_fn as usize);

    // Enable machine mode interrupts with mstatus reg.
    set_mstatus(get_mstatus() | MSTATUS_MIE);

    // Enable machine-mode timer interrupts.
    let mie = get_mie() | MIE_MTIE;
    set_mie(mie);
}
