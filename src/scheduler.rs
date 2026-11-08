use std::pin::Pin;

use crate::{context::{init_context, Context}, coroutine::{Coroutine, CoroutineState}, stack::Stack, utils::{coroutine_entry, CoroutineEntry}};

pub struct Scheduler<C, S> where C: Context, S: Stack {
    context: C,
    coroutines: Vec<Box<Coroutine<C, S>>>
}

impl<C, S> Scheduler<C, S> where C: Context, S: Stack {
    pub fn new(stack_size: usize, entry: CoroutineEntry<C>) -> Pin<Box<Self>> {
        let coroutine = Coroutine::new(stack_size, entry);

        let mut scheduler = Box::pin( Self {
            context: C::new(),
            coroutines: vec![coroutine]
        });

        let scheduler_ref = unsafe {
            Pin::get_unchecked_mut(scheduler.as_mut())
        };

        let scheduler_ctx_ptr = &raw mut scheduler_ref.context;

        for coroutine in scheduler_ref.coroutines.iter_mut() {
            let coro_ptr = (&raw mut **coroutine).cast();

            init_context(&mut coroutine.context, coroutine.stack.top(), coro_ptr, scheduler_ctx_ptr.cast(), coroutine_entry::<C, S>);
        }

        scheduler
    }

    pub fn run(&mut self) {
        let scheduler_ctx = &raw mut self.context;
        for coroutine in self.coroutines.iter_mut() {
            let coro = &raw mut **coroutine;
            unsafe {
                match (*coro).state {
                    CoroutineState::Created | CoroutineState::Suspended => {
                        Coroutine::resume(coro, scheduler_ctx);
                    },
                    CoroutineState::Running => {
                        panic!("Coroutine already running!")
                    },
                    CoroutineState::Finished => {},
                }
            }
        }
    }

    pub fn spawn(self: Pin<&mut Self>, stack_size: usize, entry: CoroutineEntry<C>) {
        let mut coroutine = Coroutine::<_,S>::new(stack_size, entry);
        let scheduler = unsafe {
            Pin::get_unchecked_mut(self)
        };
        let coro_ptr = (&raw mut *coroutine).cast();
        let scheduler_ctx_ptr = &raw mut scheduler.context;
        init_context(&mut coroutine.context, coroutine.stack.top(), coro_ptr, scheduler_ctx_ptr.cast(), coroutine_entry::<C, S>);
        scheduler.coroutines.push(coroutine);
    }
}