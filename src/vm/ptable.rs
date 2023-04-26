//! Page table
// VA: 39bits, PA: 56bits
// PTE size = 8 bytes
use crate::hw::param::*;
use crate::hw::riscv::*;
use crate::vm::*;
use core::assert;

const PAGE_SIZE: usize = 4096;
#[cfg(target_pointer_width = "32")]
const PTE_SIZE: usize = 4;
#[cfg(target_pointer_width = "64")]
const PTE_SIZE: usize = 8;
const NUM_PTES: usize = PAGE_SIZE / PTE_SIZE;

#[cfg(target_pointer_width = "32")]
const PT_LEVELS: u32 = 2;
#[cfg(target_pointer_width = "64")]
const PT_LEVELS: u32 = 3;

#[cfg(target_pointer_width = "32")]
const VA_MAX: usize = ((1u64 << 32) - 1) as usize; // Sv32
#[cfg(target_pointer_width = "64")]
const VA_MAX: usize = ((1u64 << 39) - 1) as usize; // Sv39

const PTE_VALID: usize = 1 << 0;
const PTE_READ: usize = 1 << 1;
const PTE_WRITE: usize = 1 << 2;
const PTE_EXEC: usize = 1 << 3;
const PTE_USER: usize = 1 << 4;
const PTE_GLOBAL: usize = 1 << 5;
const PTE_ACCESSED: usize = 1 << 6;
const PTE_DIRTY: usize = 1 << 7;

pub type VirtAddress = *mut usize;
pub type PhysAddress = *mut usize;
type PTEntry = usize;
/// Supervisor Address Translation and Protection.
/// Section 4.1.12 of risc-v priviliged ISA manual.
pub type SATPAddress = usize;

/// Abstraction of a page table at a physical address.
/// Notice we didn't use a rust array here, instead
/// implementing our own indexing methods, functionally
/// similar to that of an array.
#[derive(Copy, Clone)]
#[repr(C)]
pub struct PageTable {
    base: PhysAddress, // Page Table located at base address.
}

macro_rules! pte_get_flag {
    ($pte:expr, $flag:expr) => {
        ($pte) & $flag != 0
    };
}

macro_rules! pte_set_flag {
    ($pte:expr, $flag:expr) => {
        (($pte) | $flag)
    };
}

#[inline(always)]
fn vpn(ptr: VirtAddress, level: u32) -> usize {
    (ptr.addr() / PAGE_SIZE / NUM_PTES.pow(level)) % NUM_PTES
}

#[inline(always)]
fn pte_to_phy(pte: PTEntry) -> PhysAddress {
    ((pte >> 10) * PAGE_SIZE) as *mut usize
}

#[inline(always)]
fn phy_to_pte(ptr: PhysAddress) -> PTEntry {
    (ptr.addr() / PAGE_SIZE) << 10
}

#[inline(always)]
fn phy_to_satp(ptr: PhysAddress) -> usize {
    if cfg!(target_pointer_width = "32") {
        (1 << 31) | (ptr.addr() / PAGE_SIZE) // Sv32
    } else {
        (8 << 60) | (ptr.addr() / PAGE_SIZE) // Sv39
    }
}

macro_rules! page_align_down {
    ($p:expr) => {
        ($p).map_addr(|addr| addr & !(PAGE_SIZE - 1))
    };
}

// Read the memory at location self + index * PTE size
unsafe fn get_phy_offset(phy: PhysAddress, index: usize) -> *mut PTEntry {
    phy.byte_add(index * PTE_SIZE)
}

fn set_pte(pte: *mut PTEntry, contents: PTEntry) {
    unsafe {
        pte.write_volatile(contents);
    }
}

fn read_pte(pte: *mut PTEntry) -> PTEntry {
    unsafe { pte.read_volatile() }
}

impl From<PTEntry> for PageTable {
    fn from(pte: PTEntry) -> Self {
        PageTable {
            base: pte_to_phy(pte),
        }
    }
}

impl PageTable {
    fn index_mut(&self, idx: usize) -> *mut PTEntry {
        assert!(idx < NUM_PTES);
        unsafe { get_phy_offset(self.base, idx) }
    }
    pub fn write_satp(&self) {
        flush_tlb();
        set_satp(phy_to_satp(self.base));
        flush_tlb();
    }
}

// Get the address of the PTE for va given the page table pt.
// Returns Either PTE or None, callers responsibility to use PTE
// or allocate a new page.
unsafe fn walk(pt: PageTable, va: VirtAddress, alloc_new: bool) -> Result<*mut PTEntry, VmError> {
    let mut table = pt;
    assert!(va.addr() <= VA_MAX);
    for level in (1..PT_LEVELS).rev() {
        let idx = vpn(va, level);
        let next: *mut PTEntry = table.index_mut(idx);
        table = match pte_get_flag!(*next, PTE_VALID) {
            true => PageTable::from(*next),
            false => {
                if alloc_new {
                    match PAGEPOOL.get_mut().unwrap().palloc() {
                        Ok(pg) => {
                            *next = pte_set_flag!(phy_to_pte(pg.addr), PTE_VALID);
                            PageTable::from(phy_to_pte(pg.addr))
                        }
                        Err(e) => return Err(e),
                    }
                } else {
                    return Err(VmError::PallocFail);
                }
            }
        };
    }
    // Last, return PTE leaf. Assuming we are all using 4K pages right now.
    // Caller's responsibility to check flags.
    let idx = vpn(va, 0);
    Ok(table.index_mut(idx))
}

/// Helper for making flags for page_map for unpriviledged processes
pub fn user_process_flags(r: bool, w: bool, e: bool) -> usize {
    PTE_USER
        | if r { PTE_READ } else { 0 }
        | if w { PTE_WRITE } else { 0 }
        | if e { PTE_EXEC } else { 0 }
}

/// Maps some number of pages into the VM given by pt of byte length
/// size.
pub fn page_map(
    pt: PageTable,
    va: VirtAddress,
    pa: PhysAddress,
    size: usize,
    flag: usize,
) -> Result<(), VmError> {
    // Round down to page aligned boundary (multiple of pg size).
    let mut start = page_align_down!(va);
    let mut phys = pa;
    let end = page_align_down!(va.map_addr(|addr| addr + (size - 1)));

    while start <= end {
        let walk_addr = unsafe { walk(pt, start, true) };
        match walk_addr {
            Err(e) => {
                return Err(e);
            }
            Ok(pte_addr) => {
                if read_pte(pte_addr) & PTE_VALID != 0 {
                    return Err(VmError::PallocFail);
                }
                set_pte(pte_addr, pte_set_flag!(phy_to_pte(phys), flag | PTE_VALID));
                start = start.map_addr(|addr| addr + PAGE_SIZE);
                phys = phys.map_addr(|addr| addr + PAGE_SIZE);
            }
        }
    }

    Ok(())
}

/// Create the kernel page table with 1:1 mappings to physical memory.
/// First allocate a new page for the kernel page table.
/// Next, map memory mapped I/O devices to the kernel page table.
/// Then map the kernel .text, .data, .rodata and .bss sections.
/// Additionally, map a stack+guard page for each hart.
/// Finally map, the remaining physical memory to kernel virtual memory as
/// the kernel 'heap'.
pub fn kpage_init() -> Result<PageTable, VmError> {
    let base = unsafe {
        PAGEPOOL
            .get_mut()
            .unwrap()
            .palloc()
            .expect("Couldn't allocate root kernel page table.")
    };
    //log!(Debug, "Kernel page table base addr: {:#02x}", base.addr.addr());
    let kpage_table = PageTable {
        base: base.addr as *mut usize,
    };

    page_map(
        kpage_table,
        UART_BASE as *mut usize,
        UART_BASE as *mut usize,
        PAGE_SIZE,
        PTE_READ | PTE_WRITE,
    )?;
    log!(Debug, "Successfully mapped UART into kernel pgtable...");

    page_map(
        kpage_table,
        DRAM_BASE,
        DRAM_BASE as *mut usize,
        text_end().addr() - DRAM_BASE.addr(),
        PTE_READ | PTE_EXEC,
    )?;
    log!(
        Debug,
        "Succesfully mapped kernel text into kernel pgtable..."
    );

    page_map(
        kpage_table,
        text_end(),
        text_end() as *mut usize,
        rodata_end().addr() - text_end().addr(),
        PTE_READ,
    )?;
    log!(
        Debug,
        "Succesfully mapped kernel rodata into kernel pgtable..."
    );

    page_map(
        kpage_table,
        rodata_end(),
        rodata_end() as *mut usize,
        data_end().addr() - rodata_end().addr(),
        PTE_READ | PTE_WRITE,
    )?;
    log!(
        Debug,
        "Succesfully mapped kernel data into kernel pgtable..."
    );

    // This maps hart 0, 1 stack pages in opposite order as entry.S. Shouln't necessarily be a
    // problem.
    let base = stacks_start();
    for s in 0..NHART {
        let stack = unsafe { base.byte_add(PAGE_SIZE * (1 + s * 3)) };
        page_map(
            kpage_table,
            stack,
            stack,
            PAGE_SIZE * 2,
            PTE_READ | PTE_WRITE,
        )?;
        log!(
            Debug,
            "Succesfully mapped kernel stack {} into kernel pgtable...",
            s
        );
    }

    // This maps hart 0, 1 stack pages in opposite order as entry.S. Shouln't necessarily be a
    // problem.
    let base = intstacks_start();
    for i in 0..NHART {
        let m_intstack = unsafe { base.byte_add(PAGE_SIZE * (1 + i * 4)) };
        // Map hart i m-mode handler.
        page_map(
            kpage_table,
            m_intstack,
            m_intstack,
            PAGE_SIZE,
            PTE_READ | PTE_WRITE,
        )?;
        // Map hart i s-mode handler
        let s_intstack = unsafe { m_intstack.byte_add(PAGE_SIZE * 2) };
        page_map(
            kpage_table,
            s_intstack,
            s_intstack,
            PAGE_SIZE,
            PTE_READ | PTE_WRITE,
        )?;
        log!(
            Debug,
            "Succesfully mapped interrupt stack for hart {} into kernel pgtable...",
            i
        );
    }

    page_map(
        kpage_table,
        bss_start(),
        bss_start(),
        bss_end().addr() - bss_start().addr(),
        PTE_READ | PTE_WRITE,
    )?;
    log!(Debug, "Succesfully mapped kernel bss...");

    page_map(
        kpage_table,
        bss_end(),
        bss_end(),
        memory_end().addr() - bss_end().addr(),
        PTE_READ | PTE_WRITE,
    )?;
    log!(Debug, "Succesfully mapped kernel heap...");

    Ok(kpage_table)
}
