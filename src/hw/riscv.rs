//! Rust wrappers around RISC-V routines
use core::arch::asm;
use paste::paste;

// TODO: Do we really need .option norvc?

macro_rules! csr_getter {
    ($name:ident, $value:ident, $desc:tt) => {
        paste! {
            #[doc = concat!("Get ", $desc, ".")]
            pub fn [<get_ $name>]() -> usize {
                let $value: usize;
                unsafe {
                    asm!(
                        ".option norvc",
                        concat!("csrr {}, ", stringify!($name)),
                        out(reg) $value
                    );
                }
                $value
            }
        }
    };
}

macro_rules! csr_setter {
    ($name:ident, $value:ident, $desc:tt) => {
        paste! {
            #[doc = concat!("Set ", $desc, ".")]
            pub fn [<set_ $name>]($value: usize) {
                unsafe {
                    asm!(
                        ".option norvc",
                        concat!("csrw ", stringify!($name), ", {}"),
                        in(reg) $value
                    );
                }
            }
        }
    };
}

macro_rules! csr_getter_setter {
    ($name:ident, $value:ident, $desc:tt) => {
        csr_getter!($name, $value, $desc);
        csr_setter!($name, $value, $desc);
    };
}

csr_getter!(mhartid, id, "ID of current hart (M mode)");
csr_getter_setter!(mstatus, status, "status register (M mode)");
#[cfg(target_pointer_width = "32")]
csr_getter_setter!(mstatush, status, "additional status register (M mode)");
csr_getter_setter!(sstatus, status, "status register (S mode)");
csr_getter!(mcause, cause, "trap cause register (M mode)");
csr_getter!(scause, cause, "trap cause register (S mode)");
csr_getter_setter!(mepc, addr, "exception program counter (M mode)");
csr_getter_setter!(sepc, addr, "exception program counter (S mode)");
csr_getter_setter!(mip, interrupts, "interrupt-pending register (M mode)");
csr_getter_setter!(sip, interrupts, "interrupt-pending register (S mode)");
csr_getter_setter!(mie, interrupts, "interrupt-enable register (M mode)");
csr_getter_setter!(sie, interrupts, "interrupt-enable register (S mode)");
csr_getter_setter!(mscratch, scratch, "scratch register (M mode)");
csr_getter_setter!(sscratch, scratch, "scratch register (S mode)");
csr_getter_setter!(mtvec, addr, "trap-handler base address (M mode)");
csr_getter_setter!(stvec, addr, "trap-handler base address (S mode)");

csr_getter_setter!(
    satp,
    value,
    "address translation and protection register (S mode)"
);
csr_getter_setter!(
    medeleg,
    exceptions,
    "exception delegation register (M mode)"
);
csr_getter_setter!(
    mideleg,
    exceptions,
    "interrupt delegation register (M mode)"
);
csr_getter_setter!(
    pmpaddr0,
    addr,
    "physical memory protection address register 0"
);
csr_getter_setter!(
    pmpcfg0,
    config,
    "physical memory protection configuration register 0"
);

/// Just for curiosity's sake:
/// <https://github.com/rust-lang/rust/issues/82753>
/// tp := thread pointer register.
/// This way we can query a hart's hartid and store it in tp reg.
pub fn set_tp(id: usize) {
    unsafe {
        asm!("mv tp, {}", in(reg) id);
    }
}

pub fn get_tp() -> usize {
    let tp: usize;
    unsafe {
        asm!("mv {}, tp", out(reg) tp);
    }
    tp
}

/// The `zero, zero` arguments to `sfence.vma` insn mean
/// we completely flush every TLB entry for all ASIDs.
pub fn flush_tlb() {
    unsafe {
        asm!("sfence.vma zero, zero");
    }
}
