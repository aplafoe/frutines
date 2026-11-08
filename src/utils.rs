use crate::{context::{Context, CoroContext}, coroutine::{Coroutine, CoroutineHandle, CoroutineState}, stack::Stack};
use core::arch::global_asm;

global_asm!(r#"
    .global _context_switch_asm
    .align 2

_context_switch_asm:
    mov x9, sp
    str x9, [x0, #0]

    stp x19, x20, [x0, #8]
    stp x21, x22, [x0, #24]
    stp x23, x24, [x0, #40]
    stp x25, x26, [x0, #56]
    stp x27, x28, [x0, #72]
    stp x29, x30, [x0, #88]

    stp d8, d9,   [x0, #104]
    stp d10, d11, [x0, #120]
    stp d12, d13, [x0, #136]
    stp d14, d15, [x0, #152]

    ldp x19, x20, [x1, #8]
    ldp x21, x22, [x1, #24]
    ldp x23, x24, [x1, #40]
    ldp x25, x26, [x1, #56]
    ldp x27, x28, [x1, #72]
    ldp x29, x30, [x1, #88]

    ldp d8, d9,   [x1, #104]
    ldp d10, d11, [x1, #120]
    ldp d12, d13, [x1, #136]
    ldp d14, d15, [x1, #152]

    ldr x9, [x1, #0]
    mov sp, x9

    ret

    .global _coroutine_entry_trampoline_asm
    .align 2

_coroutine_entry_trampoline_asm:
    mov x0, x19
    mov x1, x20
    blr x21

    brk #0
"#);

pub type CoroutineEntry<C, T> = fn(&mut CoroutineHandle<C>) -> T;

pub(crate) unsafe extern "C" fn coroutine_entry<C, S, T>(data: *mut (), scheduler_ctx: *mut ()) -> T 
where C: Context, S: Stack {
    let coro = data.cast::<Coroutine<C, S, T>>();
    let scheduler_ctx = scheduler_ctx.cast::<C>();

    let coro_ctx = unsafe {
        &raw mut (*coro).context
    };

    let coro_state = unsafe {
        &raw mut (*coro).state
    };

    let entry = unsafe {
        (*coro).entry.take().expect("No entry")
    };

    let mut handle = CoroutineHandle {
        coroutine_context: coro_ctx,
        scheduler_context: scheduler_ctx,
        coro_state
    };

    let result = entry(&mut handle);

    unsafe {
        (*coro).result = Some(result);
        coroutine_exit(coro, scheduler_ctx.cast())
    }
}

unsafe extern "C" {
    pub(crate) fn context_switch_asm(from: *mut CoroContext, to: *const CoroContext);
    pub(crate) fn coroutine_entry_trampoline_asm();
}

pub unsafe fn context_switch<C: Context>(from: *mut C, to: *const C) {
    unsafe { context_switch_asm(from.cast(), to.cast()); }
}

pub(crate) unsafe fn coroutine_exit<C: Context, S: Stack, T>(coro: *mut Coroutine<C, S, T>, scheduler_ctx: *const C) -> ! {
    unsafe {
        (*coro).state = CoroutineState::Finished;
    }
    unsafe {
        context_switch(&mut (*coro).context, scheduler_ctx);
    }

    unreachable!()
}