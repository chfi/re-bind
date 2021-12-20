use futures::{future::RemoteHandle, task::SpawnExt, Future, FutureExt};

use crossbeam::atomic::AtomicCell;
use futures_timer::Delay;
use rustc_hash::FxHashMap;
use std::sync::Arc;

use anyhow::Result;

const LOOP_DELAY_MS: u64 = 3;



/*
#[derive(Default)]
pub struct Automaton {
    states: Vec<Vec<Option<usize>>>,
    outputs: Vec<Vec<Option<Output>>>,
}
*/

pub enum Output {
    Callback(Box<dyn Fn() + Send + Sync + 'static>),
}

#[derive(Default)]
pub struct BindMan {
    // tasks: FxHashMap<usize, Pin<Box<dyn Future<Output = ()> + Send + Sync + 'static>>
    tasks: FxHashMap<usize, RemoteHandle<()>>,
    next_id: usize,
}

// these are basically for triggering outputs, not much else
#[derive(Clone)]
pub struct TaskHandle {
    id: usize,
    counter: Arc<AtomicCell<u32>>,
}

impl TaskHandle {
    pub fn spawn(&self) {
        self.counter.fetch_add(1);
    }
}

impl BindMan {
    // pub fn spawn<F: Future<Output = ()> + Send + 'static>(
    pub fn spawn<F: Fn() + Send + 'static>(
        &mut self,
        thread_pool: &futures::executor::ThreadPool,
        f: F
    ) -> Result<TaskHandle> {
        let id = self.next_id;

        let counter = Arc::new(AtomicCell::new(0u32));
        let count = counter.clone();

        let handle = thread_pool.spawn_with_handle(async move {
            loop {
                Delay::new(std::time::Duration::from_millis(LOOP_DELAY_MS)).await;
                let val = count.load();

                if val > 0 {
                    count.fetch_sub(1u32);
                    f();
                }
            }
        })?;
        self.tasks.insert(id, handle);
        self.next_id += 1;
        Ok(TaskHandle { id, counter })
    }
}
