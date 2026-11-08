use crate::context::Context;
use crate::stack::Stack;
use crate::utils::CoroutineEntry;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CoroutineState {
    Created,
    Running,
    Suspended,
    Finished
}

#[derive(Debug)]
pub(crate) struct Coroutine<C, S> where C: Context, S: Stack {
    pub context: C,
    pub stack: S,
    pub state: CoroutineState,
    pub entry: Option<CoroutineEntry<C>>
}

pub struct CoroutineHandle<C> where C: Context {
    pub coroutine_context: *mut C,
    pub scheduler_context: *mut C,
    pub coro_state: *mut CoroutineState
}

impl<C: Context> CoroutineHandle<C> {
    pub fn yield_now(&mut self) {
        unsafe {
            *self.coro_state = CoroutineState::Suspended;
            C::switch(self.coroutine_context, &(*self.scheduler_context));
        }
    }
}

impl<C: Context, S: Stack> Coroutine<C, S> {
    pub fn new(stack_size: usize, entry: CoroutineEntry<C>) -> Box<Self> {
        let c = Box::new(Self {
            context: Context::new(),
            stack: Stack::new(stack_size),
            state: CoroutineState::Created,
            entry: Some(entry)
        });
        c
    }
    pub fn resume(this: *mut Self, scheduler_ctx: *mut C) {
        unsafe {
            (*this).state = CoroutineState::Running;

            C::switch(scheduler_ctx, &(*this).context);
        }
    }
}