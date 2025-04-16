use crate::detail::align_down;
use crate::stack::Stack;

// Remove global_asm! include. Assembly will be linked via build.rs + cc crate.

// Define the function pointer type for the entry point
// ABI is likely "C", but needs verification for Windows AArch64 specifically.
// Sticking with "C" for now as it's common.
pub type InitFn = extern "C" fn(usize, *mut usize) -> !;

// Wrapper for the generic generator entry point
pub extern "C" fn gen_init(a1: usize, a2: *mut usize) -> ! {
    super::gen::gen_init_impl(a1, a2)
}

// Declare the external assembly functions
extern "C" {
    pub fn bootstrap_green_task();
    pub fn prefetch(data: *const usize);
    pub fn swap_registers(out_regs: *mut Registers, in_regs: *const Registers);
}

// Define the structure to hold the saved registers based on Windows AArch64 ABI
#[repr(C)]
#[derive(Debug)]
pub struct Registers {
    // Callee-saved GPRs: x19-x28 (10), fp (x29), lr (x30) = 12 registers
    // SP is handled separately but needs space in the struct if saved explicitly by asm.
    // Let's allocate space conceptually.
    // Callee-saved FP/SIMD: d8-d15 (8 double-precision = 8 * 8 bytes)
    // Total size needs careful calculation based on how asm saves them.
    // Let's start with a placeholder size and refine based on asm implementation.
    // Assuming direct save: 12 GPRs * 8 bytes + SP * 8 bytes + 8 FPRs * 8 bytes = 21 * 8 = 168 bytes
    // Using a simple array for now, indices need mapping in initialize_call_frame and asm.
    pub regs: [usize; 21], // Placeholder size: 12 GPRs + SP + 8 FPRs
                           // Indices (example, needs finalization):
                           // 0-9: x19-x28
                       // 10: fp (x29)
                       // 11: lr (x30)
                       // 12: sp
                       // 13-20: d8-d15 (each takes 8 bytes)
}

impl Registers {
    pub fn new() -> Registers {
        Registers { regs: [0; 21] } // Initialize with zeros
    }

    #[inline]
    pub fn prefetch(&self) {
        // Assuming SP is stored at index 12 based on the placeholder structure
        let sp_ptr = self.regs[12] as *const usize;
        unsafe {
            // Prefetch the top of the stack
            prefetch(sp_ptr);
            prefetch(sp_ptr.add(8)); // Prefetch next cache line potentially
        }
    }
}

// Initialize the register context for a new generator
pub fn initialize_call_frame(
    regs: &mut Registers,
    fptr: InitFn,
    arg: usize,
    arg2: *mut usize,
    stack: &Stack,
) {
    // Map symbolic names to placeholder indices (adjust as needed)
    const X19: usize = 0; // Start of x19-x28 block
    const X20: usize = 1;
    const X21: usize = 2;
    const FP: usize = 10; // x29
    const LR: usize = 11; // x30
    const SP: usize = 12; // Stack Pointer

    // Get the aligned stack pointer (top of the stack)
    let sp = align_down(stack.end());

    // Store arguments and the target function pointer in temporary registers
    // These will be moved to x0, x1 by bootstrap_green_task before calling fptr
    regs.regs[X19] = arg;
    regs.regs[X20] = arg2 as usize;
    regs.regs[X21] = fptr as usize; // Target function

    // Set the frame pointer (x29) to the initial stack pointer
    regs.regs[FP] = sp as usize;

    // Set the link register (x30) to the bootstrap function
    regs.regs[LR] = bootstrap_green_task as usize;

    // Set the stack pointer (sp)
    regs.regs[SP] = sp as usize;

    // Initialize other callee-saved GPRs (x22-x28) and FP regs (d8-d15) to 0
    // Assuming indices 3-9 for x22-x28 and 13-20 for d8-d15
    for i in 3..=9 { regs.regs[i] = 0; }
    for i in 13..=20 { regs.regs[i] = 0; }

    // Note: No TIB handling for now, as discussed.
}
