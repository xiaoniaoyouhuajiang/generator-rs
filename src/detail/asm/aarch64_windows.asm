TITLE   ARM64 Context Switching for Windows (MASM)

    INCLUDE arm64unwind.inc ; Include standard unwind definitions

    .CODE ; Start code segment

; Prefetch function (simple, likely no complex unwind needed)
PUBLIC prefetch
prefetch PROC FRAME
    prfm pldl1keep, [x0]
    ret
prefetch ENDP FRAME

; Bootstrap function (also simple)
PUBLIC bootstrap_green_task
bootstrap_green_task PROC FRAME
    mov x0, x19  ; First argument
    mov x1, x20  ; Second argument
    mov x30, #0  ; Clear LR
    br x21       ; Branch to target function in x21
bootstrap_green_task ENDP FRAME


; Context switching function
PUBLIC swap_registers
swap_registers PROC FRAME ; Declare function and enable unwind info generation
    ; x0 = out_regs (pointer to save current context)
    ; x1 = in_regs (pointer to load new context)

    ; Prologue: Save non-volatile registers to the stack or specified memory.
    ; The unwind directives describe these actions.
    ; We save directly to the memory pointed by x0 (out_regs).
    ; The offsets match the Rust Registers struct.

    ; Save GPRs: x19-x28, fp (x29), lr (x30)
    stp x19, x20, [x0, #0]      ; Offset 0
    stp x21, x22, [x0, #16]     ; Offset 16
    stp x23, x24, [x0, #32]     ; Offset 32
    stp x25, x26, [x0, #48]     ; Offset 48
    stp x27, x28, [x0, #64]     ; Offset 64
    stp x29, x30, [x0, #80]     ; Offset 80 (fp, lr)

    ; Save sp
    mov x2, sp
    str x2, [x0, #96]           ; Offset 96

    ; Save d8-d15 (low 64 bits)
    stp d8,  d9,  [x0, #104]    ; Offset 104
    stp d10, d11, [x0, #120]    ; Offset 120
    stp d12, d13, [x0, #136]    ; Offset 136
    stp d14, d15, [x0, #152]    ; Offset 152

    ; --- Load new context (in_regs -> CPU registers) ---
    ; Load GPRs: x19-x28, fp (x29), lr (x30)
    ldp x19, x20, [x1, #0]      ; Offset 0
    ldp x21, x22, [x1, #16]     ; Offset 16
    ldp x23, x24, [x1, #32]     ; Offset 32
    ldp x25, x26, [x1, #48]     ; Offset 48
    ldp x27, x28, [x1, #64]     ; Offset 64
    ldp x29, x30, [x1, #80]     ; Offset 80 (fp, lr)

    ; Load sp
    ldr x2, [x1, #96]           ; Offset 96
    mov sp, x2

    ; Load d8-d15
    ldp d8,  d9,  [x1, #104]    ; Offset 104
    ldp d10, d11, [x1, #120]    ; Offset 120
    ldp d12, d13, [x1, #136]    ; Offset 136
    ldp d14, d15, [x1, #152]    ; Offset 152

    ; Epilogue: Restore registers (already done by loading) and return/branch.
    ; The unwind directives need to match the prologue's inverse.

    ; Branch to the loaded link register (lr / x30)
    br x30

swap_registers ENDP FRAME ; End function definition and unwind info

    END ; End of assembly file
