//! minimal rust kernel built for (qemu virt machine) riscv.
#![no_std]
#![no_main]
#![feature(pointer_byte_offsets)]
#![feature(error_in_core)]
#![feature(sync_unsafe_cell)]
#![feature(panic_info_message)]
#![feature(strict_provenance)]
#![feature(unsized_fn_params)]
#![allow(dead_code)]

use core::arch::asm;
use core::panic::PanicInfo;
extern crate alloc;

#[macro_use]
pub mod log;
pub mod asm;
pub mod device;
pub mod file;
pub mod hw;
pub mod lock;
pub mod process;
pub mod trap;
pub mod vm;

use crate::device::uart;
use crate::hw::param;
use crate::hw::riscv::*;

// The never type "!" means diverging function (never returns).
#[panic_handler]
fn panic(info: &PanicInfo) -> ! {
    let default = format_args!("No message provided");
    let msg = match info.message() {
        Some(msg) => msg,
        None => &default,
    };
    match info.location() {
        None => {
            println!("PANIC! {} at {}", msg, "No location provided");
        }
        Some(loc) => {
            println!("PANIC! {} at {}:{}", msg, loc.file(), loc.line());
        }
    }
    loop {}
}

/// This gets called from entry.S and runs on each hart.
/// Run configuration steps that will allow us to run the
/// kernel in supervisor mode.
#[no_mangle]
pub extern "C" fn _start() {
    // xv6-riscv/kernel/start.c
    let fn_main = main as *const ();

    // Set the *prior* privilege mode to supervisor (01).
    // mstatus[12:11] = MPP
    set_mstatus(get_mstatus() & !(1 << 12) | (1 << 11));

    // Set machine exception prog counter to
    // our main function for later mret call.
    set_mepc(fn_main as usize);

    // Disable paging while setting up.
    set_satp(0);

    // Delegate trap handlers to kernel in supervisor mode.
    // Write 1's to all bits of register and read back reg
    // to see which positions hold a 1.
    set_medeleg(0xffff);
    set_mideleg(0xffff);

    // Supervisor interrupt enable (sie SEIE, STIE, SSIE).
    set_sie(get_sie() | (1 << 9) | (1 << 5) | (1 << 1));

    // Now give sup mode access to phys mem.
    // Check 3.7.1 of riscv priv isa manual.
    set_pmpaddr0(0x3fffffffffffff_usize); // RTFM
    set_pmpcfg0(0xf); // 1st 8 bits are pmp0cfg

    // Store each hart's hartid in its tp reg for identification.
    set_tp(get_mhartid());

    // Get interrupts from clock and set mtev handler fn.
    hw::timerinit();

    // Now return to sup mode and jump to main().
    unsafe {
        asm!("mret");
    }
}

// Primary kernel bootstrap function.
// We ensure that we only initialize kernel subsystems
// one time by only doing so on hart0.
fn main() -> ! {
    // We only bootstrap on hart0.
    let id = get_tp();
    if id == 0 {
        uart::Uart::init();
        println!("{}", param::BANNER);
        log!(Info, "Bootstrapping on hart0...");
        trap::init();
        log!(Info, "Finished trap init...");
        let _ = vm::init();
        log!(Info, "Initialized the kernel page table...");
        unsafe {
            log!(Debug, "Testing page allocation and freeing...");
            vm::test_palloc();
            log!(Debug, "Testing galloc allocation and freeing...");
            vm::test_galloc();
        }
        log!(Debug, "Testing phys page extent allocation and freeing...");
        vm::test_phys_page();
        log!(
            Debug,
            "Successful phys page extent allocation and freeing..."
        );
        log!(Info, "Completed all hart0 initialization and testing...");
    } else {
        //Interrupt other harts to init kpgtable.
        trap::init();
    }

    loop {}
}
