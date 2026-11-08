use std::any::Any;

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

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct CoroutineId(usize);

impl CoroutineId {
    pub fn new(id: usize) -> Self {
        CoroutineId(id)
    }
}

pub trait CoroutineTask<C, S> {
    fn state(&self) -> CoroutineState;
    fn resume(&mut self, scheduler_ctx: *mut C);
    fn invoke_result(&mut self) -> Option<Box<dyn Any>>;
    fn id(&self) -> CoroutineId;
}

#[derive(Debug)]
pub(crate) struct Coroutine<C, S, T> where C: Context, S: Stack {
    pub id: CoroutineId,
    pub context: C,
    pub stack: S,
    pub state: CoroutineState,
    pub entry: Option<CoroutineEntry<C, T>>,
    pub result: Option<T>
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

impl<C: Context, S: Stack, T> Coroutine<C, S, T> {
    pub fn new(stack_size: usize, id: CoroutineId, entry: CoroutineEntry<C, T>) -> Box<Self> {
        let c = Box::new(Self {
            id,
            context: Context::new(),
            stack: Stack::new(stack_size),
            state: CoroutineState::Created,
            entry: Some(entry),
            result: None
        });
        c
    }
}

impl<C, S, T> CoroutineTask<C, S> for Coroutine<C, S, T>
where C: Context, S: Stack, T: 'static {
    fn state(&self) -> CoroutineState {
        self.state
    }
    fn resume(&mut self, scheduler_ctx: *mut C) {
        self.state = CoroutineState::Running;
        unsafe {
            C::switch(scheduler_ctx, &(*self).context);
        }
    }
    fn invoke_result(&mut self) -> Option<Box<dyn Any>> {
        self.result.take().map(|v| Box::new(v) as Box<dyn Any>)
    }
    fn id(&self) -> CoroutineId {
        self.id
    }
}