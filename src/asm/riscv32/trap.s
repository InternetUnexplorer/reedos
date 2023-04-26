# TODO: properly adapt this to RV32

.macro save_gp_regs
    addi sp, sp, -256

    sw x0, 0(sp)
    sw x1, 8(sp)
    sw x2, 16(sp)
    sw x3, 24(sp)
    sw x4, 32(sp)
    sw x5, 40(sp)
    sw x6, 48(sp)
    sw x7, 56(sp)
    sw x8, 64(sp)
    sw x9, 72(sp)
    sw x10, 80(sp)
    sw x11, 88(sp)
    sw x12, 96(sp)
    sw x13, 104(sp)
    sw x14, 112(sp)
    sw x15, 120(sp)
    sw x16, 128(sp)
    sw x17, 136(sp)
    sw x18, 144(sp)
    sw x19, 152(sp)
    sw x20, 160(sp)
    sw x21, 168(sp)
    sw x22, 176(sp)
    sw x23, 184(sp)
    sw x24, 192(sp)
    sw x25, 200(sp)
    sw x26, 208(sp)
    sw x27, 216(sp)
    sw x28, 224(sp)
    sw x29, 232(sp)
    sw x30, 240(sp)
.endm

.macro load_gp_regs
    lw x0, 0(sp)
    lw x1, 8(sp)
    lw x2, 16(sp)
    lw x3, 24(sp)
    lw x5, 40(sp)
    lw x6, 48(sp)
    lw x7, 56(sp)
    lw x8, 64(sp)
    lw x9, 72(sp)
    lw x10, 80(sp)
    lw x11, 88(sp)
    lw x12, 96(sp)
    lw x13, 104(sp)
    lw x14, 112(sp)
    lw x15, 120(sp)
    lw x16, 128(sp)
    lw x17, 136(sp)
    lw x18, 144(sp)
    lw x19, 152(sp)
    lw x20, 160(sp)
    lw x21, 168(sp)
    lw x22, 176(sp)
    lw x23, 184(sp)
    lw x24, 192(sp)
    lw x25, 200(sp)
    lw x26, 208(sp)
    lw x27, 216(sp)
    lw x28, 224(sp)
    lw x29, 232(sp)
    lw x30, 240(sp)

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
