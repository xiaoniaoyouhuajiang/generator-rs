// Assembly implementation for Windows AArch64 (ARM64) ABI

.text
.global _prefetch
.align 2
_prefetch:
    // Prefetch for load, targeting L1 cache, keep data
    prfm pldl1keep, [x0]
    ret

.text
.global _bootstrap_green_task
.align 2
_bootstrap_green_task:
    // Move arguments from temporary registers (x19, x20) to argument registers (x0, x1)
    // according to Windows AArch64 ABI.
    mov x0, x19  // First argument
    mov x1, x20  // Second argument

    // Clear the link register (x30) as the target function (fptr) should not return here.
    mov x30, #0

    // Branch/return to the target function pointer stored in x21.
    // 'ret x21' is often used, but 'br x21' is equivalent and perhaps clearer.
    br x21

.text
.global _swap_registers
.align 2
_swap_registers:
    // x0 = out_regs (pointer to Registers struct to save current context)
    // x1 = in_regs (pointer to Registers struct to load new context)

    // --- Save current context (CPU registers -> out_regs) ---
    // Save callee-saved GPRs: x19-x28, fp (x29), lr (x30)
    // Save sp
    // Save callee-saved FP/SIMD: d8-d15

    // Offsets are based on the placeholder Registers struct in aarch64_windows.rs
    // 0-9: x19-x28 (10 regs = 80 bytes)
    // 10: fp (x29) (8 bytes)
    // 11: lr (x30) (8 bytes)
    // 12: sp       (8 bytes)
    // 13-20: d8-d15 (8 regs = 64 bytes)
    // Total: 80 + 8 + 8 + 8 + 64 = 168 bytes (matches struct size [usize; 21])

    stp x19, x20, [x0, #0]      // Offset 0
    stp x21, x22, [x0, #16]     // Offset 16
    stp x23, x24, [x0, #32]     // Offset 32
    stp x25, x26, [x0, #48]     // Offset 48
    stp x27, x28, [x0, #64]     // Offset 64
    stp x29, x30, [x0, #80]     // Offset 80 (fp, lr)

    // Save sp
    mov x2, sp
    str x2, [x0, #96]           // Offset 96

    // Save d8-d15 (low 64 bits)
    stp d8,  d9,  [x0, #104]    // Offset 104
    stp d10, d11, [x0, #120]    // Offset 120
    stp d12, d13, [x0, #136]    // Offset 136
    stp d14, d15, [x0, #152]    // Offset 152

    // --- Load new context (in_regs -> CPU registers) ---
    // Load callee-saved GPRs: x19-x28, fp (x29), lr (x30)
    // Load sp
    // Load callee-saved FP/SIMD: d8-d15

    ldp x19, x20, [x1, #0]      // Offset 0
    ldp x21, x22, [x1, #16]     // Offset 16
    ldp x23, x24, [x1, #32]     // Offset 32
    ldp x25, x26, [x1, #48]     // Offset 48
    ldp x27, x28, [x1, #64]     // Offset 64
    ldp x29, x30, [x1, #80]     // Offset 80 (fp, lr)

    // Load sp
    ldr x2, [x1, #96]           // Offset 96
    mov sp, x2

    // Load d8-d15
    ldp d8,  d9,  [x1, #104]    // Offset 104
    ldp d10, d11, [x1, #120]    // Offset 120
    ldp d12, d13, [x1, #136]    // Offset 136
    ldp d14, d15, [x1, #152]    // Offset 152

    // Branch to the loaded link register (lr / x30)
    br x30
