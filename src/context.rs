use crate::utils::{context_switch_asm, coroutine_entry_trampoline_asm};

pub trait Context {
    fn new() -> Self;

    fn stack_ptr(&mut self) -> &mut usize;
    fn data_ptr(&mut self) -> &mut usize;
    fn scheduler_ctx_ptr(&mut self) -> &mut usize;
    fn fn_entry_ptr(&mut self) -> &mut usize;
    fn fn_bl_ptr(&mut self) -> &mut usize;

    unsafe fn switch(from: *mut Self, to: *const Self);
}

#[repr(C)]
#[derive(Default, Debug)]
pub struct CoroContext {
    sp: usize,

    x19: usize,
    x20: usize,
    x21: usize,
    x22: usize,
    x23: usize,
    x24: usize,
    x25: usize,
    x26: usize,
    x27: usize,
    x28: usize,
    x29: usize,

    x30: usize,

    d8: usize,
    d9: usize,
    d10: usize,
    d11: usize,
    d12: usize,
    d13: usize,
    d14: usize,
    d15: u64
}

impl Context for CoroContext {
    fn new() -> Self {
        Self::default()
    }
    fn stack_ptr(&mut self) -> &mut usize {
        &mut self.sp
    }
    fn data_ptr(&mut self) -> &mut usize {
        &mut self.x19
    }
    fn scheduler_ctx_ptr(&mut self) -> &mut usize {
        &mut self.x20
    }
    fn fn_entry_ptr(&mut self) -> &mut usize {
        &mut self.x21
    }
    fn fn_bl_ptr(&mut self) -> &mut usize {
        &mut self.x30
    }
    unsafe fn switch(from: *mut Self, to: *const Self) {
        unsafe {
            context_switch_asm(from, to);
        }
    }
}

pub fn init_context<C: Context, T>(ctx: *mut C, stack: *mut u8, data: *mut (), scheduler_ctx: *mut (), entry: unsafe extern "C" fn(*mut (), *mut ()) -> T) {
    unsafe {
        let ctx = &mut *ctx;
        *ctx.stack_ptr() = stack as usize & !0xf;
        *ctx.data_ptr() = data as usize;
        *ctx.scheduler_ctx_ptr() = scheduler_ctx as usize;
        *ctx.fn_entry_ptr() = entry as usize;
        *ctx.fn_bl_ptr() = coroutine_entry_trampoline_asm as usize;
    }
}