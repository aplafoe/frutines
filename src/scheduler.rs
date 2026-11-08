use std::{any::Any, collections::HashMap, marker::PhantomData, pin::Pin};

use crate::{context::{init_context, Context}, coroutine::{Coroutine, CoroutineId, CoroutineState, CoroutineTask}, stack::Stack, utils::{coroutine_entry, CoroutineEntry}};

pub struct JoinHandle<T> {
    id: CoroutineId,
    _m: PhantomData<fn() -> T>
}


impl<T: 'static> JoinHandle<T> {
    pub fn join<C: Context + 'static, S: Stack + 'static>(self, mut scheduler: Pin<&mut Scheduler<C, S>>) -> Option<T> {
        let scheduler = unsafe {
            Pin::get_unchecked_mut(scheduler.as_mut())
        };
        while !scheduler.results.contains_key(&self.id) {
            scheduler.run();
        }

        let result = scheduler.results.remove(&self.id)?;
        result.downcast::<T>().ok().map(|v| *v)

    }
}

pub struct Scheduler<C, S> where C: Context + 'static, S: Stack + 'static {
    context: C,
    coroutines: Vec<Box<dyn CoroutineTask<C, S>>>,
    results: HashMap<CoroutineId, Box<dyn Any>>,
    next_id: usize
}

impl<C, S> Scheduler<C, S> where C: Context + 'static, S: Stack + 'static  {
    pub fn new() -> Pin<Box<Self>> {
        Box::pin(Self {
            context: C::new(),
            coroutines: vec![],
            results: HashMap::new(),
            next_id: 0,
        })
    }

    pub fn new_with<T: 'static>(stack_size: usize, entry: CoroutineEntry<C, T>) -> Pin<Box<Self>> {
        let mut coroutine = Coroutine::<C, S, T>::new(stack_size, CoroutineId::new(0), entry);

        let mut scheduler = Box::pin( Self {
            context: C::new(),
            coroutines: vec![],
            results: HashMap::new(),
            next_id: 0,
        });

        let scheduler_ref = unsafe {
            Pin::get_unchecked_mut(scheduler.as_mut())
        };

        let scheduler_ctx_ptr = &raw mut scheduler_ref.context;

        let coro_ptr = (&raw mut *coroutine).cast();
        init_context(&mut coroutine.context, coroutine.stack.top(), coro_ptr, scheduler_ctx_ptr.cast(), coroutine_entry::<C, S, T>);

        scheduler_ref.coroutines.push(coroutine);
        scheduler
    }

    pub fn run(&mut self) {
        let scheduler_ctx = &raw mut self.context;
        for coroutine in self.coroutines.iter_mut() {
            match coroutine.state() {
                CoroutineState::Created | CoroutineState::Suspended => {
                    coroutine.resume(scheduler_ctx);
                    if coroutine.state() == CoroutineState::Finished && let Some(result) = coroutine.invoke_result() {
                        self.results.insert(coroutine.id(), result);
                    }
                },
                CoroutineState::Running => {
                    panic!("Coroutine already running!")
                },
                CoroutineState::Finished => {
                    if let Some(r) = coroutine.invoke_result() {
                        self.results.insert(coroutine.id(), r);
                    }
                },
            }
        }

        //self.coroutines.retain(|c| c.state() != CoroutineState::Finished);
    }

    pub fn next_coroutine_id(&mut self) -> CoroutineId {
        let id = CoroutineId::new(self.next_id);
        self.next_id += 1;
        id
    }

    pub fn spawn<T: 'static>(self: Pin<&mut Self>, stack_size: usize, entry: CoroutineEntry<C, T>) -> JoinHandle<T> {
        let scheduler = unsafe {
            Pin::get_unchecked_mut(self)
        };
        let id = scheduler.next_coroutine_id();
        let mut coroutine = Coroutine::<_, S, T>::new(stack_size, id, entry);
        let coro_ptr = (&raw mut *coroutine).cast();
        let scheduler_ctx_ptr = &raw mut scheduler.context;
        init_context(&mut coroutine.context, coroutine.stack.top(), coro_ptr, scheduler_ctx_ptr.cast(), coroutine_entry::<C, S, T>);
        scheduler.coroutines.push(coroutine);
        JoinHandle { id: id, _m: PhantomData }
    }

    pub fn invoke_result_of(&self, id: CoroutineId) -> Option<&dyn Any> {
        self.results.get(&id).map(Box::as_ref)
    }

    pub fn run_until_complete(&mut self) {
        while !self.coroutines.is_empty() {
            self.run();
        }
    }

    pub fn is_empty(&self) -> bool {
        self.coroutines.is_empty()
    }

    pub fn len(&self) -> usize {
        self.coroutines.len()
    }
}