#[cfg(target_pointer_width = "32")]
mod riscv32 {
    use core::arch::global_asm;
    global_asm!(include_str!("asm/riscv32/entry.s"));
    global_asm!(include_str!("asm/riscv32/trap.s"));
    global_asm!(include_str!("asm/riscv32/trampoline.s"));
}

#[cfg(target_pointer_width = "64")]
mod riscv64 {
    use core::arch::global_asm;
    global_asm!(include_str!("asm/riscv64/entry.s"));
    global_asm!(include_str!("asm/riscv64/trap.s"));
    global_asm!(include_str!("asm/riscv64/trampoline.s"));
}
