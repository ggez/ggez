//! Coroutine structures,
//! allowing you to run async code without using futures.
//!
//! Useful for loading assets

use std::{
    future::{Future, IntoFuture},
    pin::Pin,
    sync::Arc,
    task::{Poll, Waker},
};

enum CoroutineState<T> {
    Future(Pin<Box<dyn Future<Output = T> + 'static>>),
    Finished,
}

/// Coroutine structure
#[allow(missing_debug_implementations)]
pub struct Coroutine<T = ()> {
    waker: Waker,
    state: CoroutineState<T>,
}

impl<T> Coroutine<T> {
    /// Constructs a new coroutine
    pub fn new(fut: impl IntoFuture<Output = T> + 'static) -> Self {
        struct Inner;
        impl std::task::Wake for Inner {
            fn wake(self: Arc<Self>) {}
        }

        let waker = Waker::from(Arc::new(Inner));

        Self {
            waker,
            state: CoroutineState::Future(Box::pin(fut.into_future())),
        }
    }

    /// Advances and possibly returns a value from the coroutine.
    pub fn poll(&mut self) -> Option<T> {
        match &mut self.state {
            // If the future isn't done, poll it
            CoroutineState::Future(fut) => {
                let mut context = std::task::Context::from_waker(&self.waker);
                match fut.as_mut().poll(&mut context) {
                    // If the future finished, return the value and set the coroutine to a finished state
                    Poll::Ready(v) => {
                        self.state = CoroutineState::Finished;
                        Some(v)
                    }
                    Poll::Pending => None,
                }
            }
            CoroutineState::Finished => None,
        }
    }
}

struct YieldOp(bool);

impl Future for YieldOp {
    type Output = ();

    fn poll(
        mut self: std::pin::Pin<&mut Self>,
        _cx: &mut std::task::Context<'_>,
    ) -> std::task::Poll<Self::Output> {
        if !self.0 {
            self.0 = true;
            return std::task::Poll::Pending;
        }

        std::task::Poll::Ready(())
    }
}

/// Wait 1 poll before finishing.
/// Useful for making infinite coroutines without blocking [`Coroutine::poll`] forever
pub fn yield_now() -> impl Future<Output = ()> {
    YieldOp(false)
}
