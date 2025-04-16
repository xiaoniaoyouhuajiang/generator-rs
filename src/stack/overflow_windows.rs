// Conditionally import guard only when needed (currently x86_64)
#[cfg(target_arch = "x86_64")]
use crate::rt::guard;
use crate::rt::{Context, ContextStack}; // Keep these imports
use std::sync::Once;
// Import STATUS_GUARD_PAGE_VIOLATION instead of EXCEPTION_STACK_OVERFLOW
use windows::Win32::Foundation::STATUS_GUARD_PAGE_VIOLATION;
use windows::Win32::System::Diagnostics::Debug::{
    AddVectoredExceptionHandler, CONTEXT, EXCEPTION_POINTERS,
};

unsafe extern "system" fn vectored_handler(exception_info: *mut EXCEPTION_POINTERS) -> i32 {
    const EXCEPTION_CONTINUE_SEARCH: i32 = 0;
    const EXCEPTION_CONTINUE_EXECUTION: i32 = -1;

    let info = &*exception_info;
    let rec = &(*info.ExceptionRecord);
    eprintln!("[vectored_handler] Entered!"); // Log handler entry

    eprintln!("[vectored_handler] Entered!"); // Log handler entry

    eprintln!("[vectored_handler] Entered!"); // Log handler entry

    let context = &mut (*info.ContextRecord);

    // Calculate is_overflow within architecture-specific blocks
    let is_overflow = {
        #[cfg(target_arch = "x86_64")]
        {
            // Calculate and use sp_match only for x86_64
            let sp_match = guard::current().contains(&(context.Rsp as usize));
            eprintln!("[vectored_handler] ExceptionCode: {:?}, (sp_match: {})", rec.ExceptionCode, sp_match);
            rec.ExceptionCode == STATUS_GUARD_PAGE_VIOLATION && sp_match
        }
        #[cfg(target_arch = "aarch64")]
        {
            // Only check exception code for aarch64
            eprintln!("[vectored_handler] ExceptionCode: {:?}", rec.ExceptionCode);
            rec.ExceptionCode == STATUS_GUARD_PAGE_VIOLATION
        }
        #[cfg(not(any(target_arch = "x86_64", target_arch = "aarch64")))]
        {
            eprintln!("[vectored_handler] ExceptionCode: {:?}", rec.ExceptionCode);
            unimplemented!("Stack overflow handling not implemented for this architecture");
            false
        }
    };

    if is_overflow
    {
        eprintln!("[vectored_handler] Guard page violation detected (assuming generator stack overflow)!"); // Log detection
        eprintln!(
            "\ncoroutine in thread '{}' has overflowed its stack\n",
            std::thread::current().name().unwrap_or("<unknown>")
        );

        let env = ContextStack::current();
        let cur = env.top();
        cur.err = Some(Box::new(crate::Error::StackErr));

        eprintln!("[vectored_handler] Calling context_init..."); // Log before call
        context_init(env.pop_context(cur as *mut _), context);
        eprintln!("[vectored_handler] Returned from context_init."); // Log after call

        //yield_now();

        EXCEPTION_CONTINUE_EXECUTION
    } else {
        EXCEPTION_CONTINUE_SEARCH
    }
}

unsafe fn init() {
    eprintln!("[overflow_windows] Registering vectored handler..."); // Log registration attempt
    let handle = AddVectoredExceptionHandler(1, Some(vectored_handler));
    if handle.is_null() {
        eprintln!("[overflow_windows] Failed to register vectored handler!");
    } else {
        eprintln!("[overflow_windows] Vectored handler registered.");
        // In a real application, you might want to store and remove the handle later.
    }
}

pub fn init_once() {
    static INIT_ONCE: Once = Once::new();

    INIT_ONCE.call_once(|| unsafe {
        init();
    })
}

#[cfg(target_arch = "x86_64")]
unsafe fn context_init(parent: &mut Context, context: &mut CONTEXT) {
    let [rbx, rsp, rbp, _, r12, r13, r14, r15, _, _, _, stack_base, stack_limit, dealloc_stack, ..] =
        parent.regs.regs.gpr;

    let rip = *(rsp as *const usize);
    let rsp = rsp + std::mem::size_of::<usize>();

    context.Rbx = rbx as u64;
    context.Rsp = rsp as u64;
    context.Rbp = rbp as u64;
    context.R12 = r12 as u64;
    context.R13 = r13 as u64;
    context.R14 = r14 as u64;
    context.R15 = r15 as u64;
    context.Rip = rip as u64;

    let teb: usize;

    unsafe {
        std::arch::asm!(
        "mov {0}, gs:[0x30]",
        out(reg) teb
        );
    }

    *((teb + 0x08) as *mut usize) = stack_base;
    *((teb + 0x10) as *mut usize) = stack_limit;
    *((teb + 0x1478) as *mut usize) = dealloc_stack;
}

// Implementation for ARM64
#[cfg(target_arch = "aarch64")]
unsafe fn context_init(parent: &mut Context, context: &mut CONTEXT) {
    // Extract saved registers from the parent context's Registers struct.
    // Indices match the layout defined in src/detail/aarch64_windows.rs
    // 0-9: x19-x28
    // 10: fp (x29)
    // 11: lr (x30) -> This becomes the new PC
    // 12: sp
    // 13-20: d8-d15
    // Access the inner array: parent.regs (RegContext) -> .regs (Registers) -> .regs ([usize; N])
    let saved_regs_array = &parent.regs.regs.regs;

    // Restore GPRs (x19-x28, fp) using the correct nested path
    context.Anonymous.Anonymous.X19 = saved_regs_array[0] as u64;
    context.Anonymous.Anonymous.X20 = saved_regs_array[1] as u64;
    context.Anonymous.Anonymous.X21 = saved_regs_array[2] as u64;
    context.Anonymous.Anonymous.X22 = saved_regs_array[3] as u64;
    context.Anonymous.Anonymous.X23 = saved_regs_array[4] as u64;
    context.Anonymous.Anonymous.X24 = saved_regs_array[5] as u64;
    context.Anonymous.Anonymous.X25 = saved_regs_array[6] as u64;
    context.Anonymous.Anonymous.X26 = saved_regs_array[7] as u64;
    context.Anonymous.Anonymous.X27 = saved_regs_array[8] as u64;
    context.Anonymous.Anonymous.X28 = saved_regs_array[9] as u64;
    context.Anonymous.Anonymous.Fp = saved_regs_array[10] as u64; // Frame Pointer (x29)

    // Restore Stack Pointer
    context.Sp = saved_regs_array[12] as u64;

    // Restore Program Counter from saved Link Register (x30)
    context.Pc = saved_regs_array[11] as u64; // Link Register (x30) value

    // Restore FP/SIMD registers (d8-d15)
    // Correct path for FP/SIMD registers (d8-d15) in CONTEXT for ARM64.
    // They are typically stored as the lower 64 bits (D[0]) of V[8] through V[15].
    let fp_regs_src_ptr = saved_regs_array.as_ptr().add(13); // Pointer to the start of saved d8-d15 data (index 13 in our array)

    // Get a mutable pointer to the start of the V array (ARM64_NT_NEON128) directly under CONTEXT.
    let v_array_ptr = context.V.as_mut_ptr();

    // Copy d8-d15 data into the lower 64 bits (D[0]) of V[8]-V[15]
    for i in 0..8 {
        // Calculate source pointer (saved_regs_array[13+i])
        let src_ptr = fp_regs_src_ptr.add(i) as *const u64;
        // Calculate destination pointer (context.V[8+i].D[0])
        // Access V[8+i], then its D field (which is [f64; 2]), then the first element D[0].
        // We need a mutable pointer to the f64, then cast it to u64 for the copy.
        let dst_ptr = unsafe { (*v_array_ptr.add(8 + i)).D.as_mut_ptr().cast::<u64>() }; // Pointer to V[8+i].D[0] as u64
        std::ptr::copy_nonoverlapping(src_ptr, dst_ptr, 1); // Copy 1 * u64
    }


    // NOTE: We are intentionally omitting the TEB/TIB stack limit manipulation
    // present in the x86_64 version for the initial ARM64 implementation.
}
