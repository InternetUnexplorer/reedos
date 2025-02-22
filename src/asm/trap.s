.macro save_gp_regs
    addi sp, sp, -256

    sd x0, 0(sp)
    sd x1, 8(sp)
    sd x2, 16(sp)
    sd x3, 24(sp)
    sd x4, 32(sp)
    sd x5, 40(sp)
    sd x6, 48(sp)
    sd x7, 56(sp)
    sd x8, 64(sp)
    sd x9, 72(sp)
    sd x10, 80(sp)
    sd x11, 88(sp)
    sd x12, 96(sp)
    sd x13, 104(sp)
    sd x14, 112(sp)
    sd x15, 120(sp)
    sd x16, 128(sp)
    sd x17, 136(sp)
    sd x18, 144(sp)
    sd x19, 152(sp)
    sd x20, 160(sp)
    sd x21, 168(sp)
    sd x22, 176(sp)
    sd x23, 184(sp)
    sd x24, 192(sp)
    sd x25, 200(sp)
    sd x26, 208(sp)
    sd x27, 216(sp)
    sd x28, 224(sp)
    sd x29, 232(sp)
    sd x30, 240(sp)
.endm

.macro load_gp_regs
    ld x0, 0(sp)
    ld x1, 8(sp)
    ld x2, 16(sp)
    ld x3, 24(sp)
    ld x5, 40(sp)
    ld x6, 48(sp)
    ld x7, 56(sp)
    ld x8, 64(sp)
    ld x9, 72(sp)
    ld x10, 80(sp)
    ld x11, 88(sp)
    ld x12, 96(sp)
    ld x13, 104(sp)
    ld x14, 112(sp)
    ld x15, 120(sp)
    ld x16, 128(sp)
    ld x17, 136(sp)
    ld x18, 144(sp)
    ld x19, 152(sp)
    ld x20, 160(sp)
    ld x21, 168(sp)
    ld x22, 176(sp)
    ld x23, 184(sp)
    ld x24, 192(sp)
    ld x25, 200(sp)
    ld x26, 208(sp)
    ld x27, 216(sp)
    ld x28, 224(sp)
    ld x29, 232(sp)
    ld x30, 240(sp)

    addi sp, sp, 256
.endm

    .section .text
# This is the machine mode trap vector(not really). It exists
# to get us into the rust handler
    .option norvc
    .align 4
    .global __mtrapvec
__mtrapvec:
    csrrw sp, mscratch, sp
    save_gp_regs

    .extern m_handler
    call m_handler

    load_gp_regs
    csrrw sp, mscratch, sp
    mret

# This is the supervisor trap vector, it just exists to get
# us into the rust handler elsewhere
    .option norvc
    .align 4
    .globl __strapvec
 __strapvec:
    csrrw sp, sscratch, sp
    save_gp_regs

    .extern s_handler
    call s_handler

    load_gp_regs
    csrrw sp, sscratch, sp
    sret
