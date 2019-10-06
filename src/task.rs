//! Asynchronous task and futures support.

use std::cell::RefCell;
use std::collections::VecDeque;
use std::future::Future;
use std::rc::{Rc, Weak as RcW};
use std::sync::{Arc, Mutex, Weak as ArcW};
use futures::channel::oneshot;
use futures::executor::{LocalPool, LocalSpawner, ThreadPool};
use futures::future::{self, FutureObj, LocalFutureObj};
use futures::task::LocalSpawn;
use futures::task::Spawn;

use crate::context::{Context, ContextKind, MainContextHandle, PoolContextHandle};
use crate::error::{GameError, GameResult};

/// A structure that stores asynchronous task-related state, including the executor used to drive
/// futures to completion on the main game thread.
pub struct TaskContext {
    main_thread_executor: Rc<RefCell<LocalPool>>,
    thread_pool_executor: Arc<RefCell<ThreadPool>>,

    // The sync and unsync queues are so that on the main thread we don't have to pay the cost
    // of syncing and locking the queue. Need feedback on whether increased code complexity is worth
    // the benefit.

    sync_with_main_unsync: Rc<RefCell<VecDeque<Box<dyn for<'ctx> FnOnce(&'ctx mut Context) + 'static>>>>,
    // TODO: Remove RefCell, its redundant with Mutex
    sync_with_main_sync: Arc<Mutex<RefCell<VecDeque<Box<dyn for<'ctx> FnOnce(&'ctx mut Context) + 'static>>>>>,
    sync_with_main_accum: Vec<Box<dyn for<'ctx> FnOnce(&'ctx mut Context) + 'static>>,

    // This is a very primitive update tracker which ticks down a count for each SleepUpdates on
    // every update tick, with O(1) inserts but O(queue_size) work per update. A smarter data
    // structure should be able to do O(num_expiring_items) work per update instead, possibly at the
    // cost of O(log(queue_size)) inserts.
    sleep_updates_unsync: Rc<RefCell<VecDeque<SleepUpdates>>>,
    sleep_updates_sync: Arc<Mutex<VecDeque<SleepUpdates>>>,
}

impl TaskContext {
    /// Creates a new [TaskContext](self::TaskContext).
    pub fn new() -> GameResult<Self> {
        Ok(Self {
            main_thread_executor: Rc::new(RefCell::new(LocalPool::new())),
            thread_pool_executor: Arc::new(RefCell::new(ThreadPool::new()?)),
            sync_with_main_unsync: Rc::new(RefCell::new(VecDeque::new())),
            sync_with_main_sync: Arc::new(Mutex::new(RefCell::new(VecDeque::new()))),
            sync_with_main_accum: Vec::new(),
            sleep_updates_unsync: Rc::new(RefCell::new(VecDeque::new())),
            sleep_updates_sync: Arc::new(Mutex::new(VecDeque::new()))
        })
    }

    /// TODO
    pub fn main_handle(&mut self) -> MainTaskHandle {
        MainTaskHandle {
            spawner: self.main_thread_executor.borrow().spawner(),
            sync_with_main_unsync: Rc::downgrade(&self.sync_with_main_unsync),
            sleep_updates_unsync: Rc::downgrade(&self.sleep_updates_unsync),
        }
    }

    /// TODO
    pub fn pool_handle(&mut self) -> PoolTaskHandle {
        PoolTaskHandle {
            thread_pool: Arc::downgrade(&self.thread_pool_executor),
            sync_with_main_sync: Arc::downgrade(&self.sync_with_main_sync),
            sleep_updates_sync: Arc::downgrade(&self.sleep_updates_sync),
        }
    }
}

/// Update the asynchronous task context before a game state update. Because `ggez::event::run`
/// cals this, most users will never need to call this themselves unless they're replacing the
/// `run` function's main loop.
pub(crate) fn tick_pre_update(ctx: &mut Context) {
    // Invoke all of the sync_with_main callbacks. We drain the callbacks out of the VecDeques
    // into a Vec so that we can pass the full Context down into the callbacks. Otherwise if we
    // iterated over the VecDeques directly then the Context would already be borrowed.
    let mut accum = std::mem::replace(&mut ctx.task_context.sync_with_main_accum, Vec::with_capacity(0));
    accum.extend(ctx.task_context.sync_with_main_unsync
        .borrow_mut()
        .drain(..));
    accum.extend(ctx.task_context.sync_with_main_sync
        .lock()
        .expect("Failed to acquire sync_with_main_sync lock - should this ever happen???")
        .borrow_mut()
        .drain(..));
    for callback in accum.drain(..) {
        callback(ctx);
    }
    let _ = std::mem::replace(&mut ctx.task_context.sync_with_main_accum, accum);

    // Tick the main thread executor before every update invocation.
    ctx.task_context.main_thread_executor
        .borrow_mut()
        .run_until_stalled();
}

/// Update the asynchronous task context after a game state update. Because `ggez::event::run`
/// cals this, most users will never need to call this themselves unless they're replacing the
/// `run` function's main loop.
pub(crate) fn tick_post_update(ctx: &mut Context) {
    // We'd be able to do all this in one pass over each VecDeque if retain_mut or drain_filter existed :(
    let mut sleep_updates_unsync = ctx.task_context.sleep_updates_unsync.borrow_mut();
    sleep_updates_unsync.iter_mut().for_each(|sleep_updates| {
        sleep_updates.updates_remaining = sleep_updates.updates_remaining.saturating_sub(1);
        if sleep_updates.updates_remaining == 0 {
            if let Some(tx) = sleep_updates.tx.take() {
                let _ = tx.send(());
            }
        }
    });
    sleep_updates_unsync.retain(|sleep_updates| sleep_updates.updates_remaining > 0);

    let mut sleep_updates_sync = ctx.task_context.sleep_updates_sync
        .lock()
        .expect("Failed to acquire sleep_updates_sync lock - should this ever happen???");
    sleep_updates_sync.iter_mut().for_each(|sleep_updates| {
        sleep_updates.updates_remaining = sleep_updates.updates_remaining.saturating_sub(1);
        if sleep_updates.updates_remaining == 0 {
            if let Some(tx) = sleep_updates.tx.take() {
                let _ = tx.send(());
            }
        }
    });
    sleep_updates_sync.retain(|sleep_updates| sleep_updates.updates_remaining > 0);
    // Eagerly unlock in case we add some code after this in the future.
    std::mem::drop(sleep_updates_unsync);
}

/// Handle to a `TaskContext` that is not thread-safe and so can only be used from the main thread.
pub struct MainTaskHandle {
    spawner: LocalSpawner,
    sync_with_main_unsync: RcW<RefCell<VecDeque<Box<dyn for<'ctx> FnOnce(&'ctx mut Context) + 'static>>>>,
    sleep_updates_unsync: RcW<RefCell<VecDeque<SleepUpdates>>>
}

/// Handle to a `TaskContext` that is thread-safe and so can be used from thread pool threads.
pub struct PoolTaskHandle {
    thread_pool: ArcW<RefCell<ThreadPool>>,
    sync_with_main_sync: ArcW<Mutex<RefCell<VecDeque<Box<dyn for<'ctx> FnOnce(&'ctx mut Context) + 'static>>>>>,
    sleep_updates_sync: ArcW<Mutex<VecDeque<SleepUpdates>>>
}

/// Spawns a future that will run to completion on the main thread. The benefit of spawning on the
/// main thread (as opposed to e.g. a thread pool) is that the future is not required to implement
/// Send. This means that the future is able to hold non-synchronized references to the game state,
/// avoiding the overhead of atomics or locking. The downside is that the future's processing time
/// counts against the game's frame time, so you need to be careful to ensure that the future does
/// not take long to execute each frame.
pub fn spawn_on_main(ctx: &mut Context, future: impl Future<Output = ()> + 'static) -> GameResult<()> {
    let local_future_obj = LocalFutureObj::new(Box::new(future));
    ctx.task_context.main_thread_executor.borrow().spawner().spawn_local_obj(local_future_obj)?;
    Ok(())
}

/// Spawns a future that will run to completion on a thread pool thread. The benefit of spawning on
/// the thread pool is that the future's work cannot block the main thread. The downside is that
/// many ggez functions can only be called on the main thread, so in order to update the game state
/// the future will need to use either `sync_with_main` or your own custom synchronization
/// strategy (for example, you might choose to store thread-safe data structures in your main game
/// state, allowing you to update them directy from this thread pool future).
pub fn spawn_on_pool(ctx: &mut Context, future: impl Future<Output = ()> + Send + 'static) -> GameResult<()> {
    let future_obj = FutureObj::new(Box::new(future));
    ctx.task_context.thread_pool_executor.borrow_mut().spawn_obj(future_obj)?;
    Ok(())
}

/// TODO
pub fn sync_with_main<'ctx, F, T>(context: impl Into<ContextKind<'ctx>>, callback: F)
        -> impl Future<Output = T>
where
    F: FnOnce(&mut Context) -> T,
    F: 'static,
    T: 'static {
    let fut = match context.into() {
        ContextKind::Real(ctx) => {
            // future::ready() would be cheaper than oneshot::channel, but is a lot more difficult
            // to get working given that the other match arms are returning Receiver<T>. A uniform
            // type makes the code much simpler.
            let (tx, rx) = oneshot::channel::<T>();
            let result: T = callback(ctx);
            let _ = tx.send(result);
            rx
        },
        ContextKind::Main(ctx) => {
            let (tx, rx) = oneshot::channel::<T>();
            ctx.task_handle.sync_with_main_unsync.upgrade()
                .expect("Cannot sync_with_main from main-thread executor because context has been dropped")
                .borrow_mut()
                .push_back(Box::new(move |main_ctx| {
                    let result: T = callback(main_ctx);
                    let _ = tx.send(result);
                }));
            rx
        },
        ContextKind::Pool(ctx) => {
            let (tx, rx) = oneshot::channel::<T>();
            ctx.task_handle.sync_with_main_sync.upgrade()
                .expect("Cannot sync_with_main from thread-pool executor because context has been dropped")
                .lock()
                .expect("Failed to acquire sync_with_main_sync lock - should this ever happen???")
                .borrow_mut()
                .push_back(Box::new(move |main_ctx| {
                    let result: T = callback(main_ctx);
                    let _ = tx.send(result);
                }));
            rx
        }
    };

    // TODO: expose cancelled case to user?
    async { fut.await.expect("sync_with_main future was dropped before completing and then polled later") }
}

/// TODO
pub fn sleep_updates<'ctx>(context: impl Into<ContextKind<'ctx>>, updates_count: usize) -> impl Future<Output = ()> {
    let (tx, rx) = oneshot::channel::<()>();
    let sleep_updates = SleepUpdates {
        updates_remaining: updates_count,
        tx: Some(tx)
    };

    match context.into() {
        ContextKind::Real(ctx) => {
            ctx.task_context.sleep_updates_unsync.borrow_mut().push_back(sleep_updates);
        },
        ContextKind::Main(ctx) => {
            ctx.task_handle.sleep_updates_unsync.upgrade()
                .expect("Cannot sleep_updates from main-thread executor because context has been dropped")
                .borrow_mut()
                .push_back(sleep_updates);
        },
        ContextKind::Pool(ctx) => {
            ctx.task_handle.sleep_updates_sync.upgrade()
                .expect("Cannot sleep_updates from thread-pool executor because context has been dropped")
                .lock()
                .expect("Failed to acquire sleep_updates_sync lock - should this ever happen???")
                .push_back(sleep_updates);
        }
    }

    // TODO: expose cancelled case to user?
    async { rx.await.expect("sleep_updates future was dropped before completing and then polled later") }
}

// This is an extremely primitive approach to tracking update ticks. The updates_remaining field is
// decremented after every update until it hits 0 at which point the tx field is sent.
struct SleepUpdates {
    updates_remaining: usize,
    tx: Option<oneshot::Sender<()>>
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn pool_task_handle_is_send_sync() {
        fn is_send_sync<T: Send + Sync>() -> bool {
            true
        }
        assert!(is_send_sync::<PoolTaskHandle>());
    }
}