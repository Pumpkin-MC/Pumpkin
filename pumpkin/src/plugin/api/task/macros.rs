#[macro_export]
macro_rules! run_task_later {
    ($server:expr, $delay_ticks:expr, $body:block) => {{
        use async_trait::async_trait;
        use std::future::Future;
        use std::pin::Pin;
        use std::sync::{
            Arc,
            atomic::{AtomicBool, Ordering},
        };
        use tokio::sync::Mutex;
        use $crate::plugin::api::task::{ScheduledHandle, TaskHandler};

        struct InlineOnceHandler {
            cancel_flag: Arc<AtomicBool>,
            future: Mutex<Option<Pin<Box<dyn Future<Output = ()> + Send>>>>,
        }

        #[async_trait]
        impl TaskHandler for InlineOnceHandler {
            async fn run(&self) {
                if self.cancel_flag.load(Ordering::Relaxed) {
                    return;
                }

                let fut = {
                    let mut guard = self.future.lock().await;
                    guard.take()
                };

                if let Some(fut) = fut {
                    fut.await;
                }
            }

            async fn cancel(&self) {
                self.cancel_flag.store(true, Ordering::Relaxed);
            }
        }

        let cancel_flag = Arc::new(AtomicBool::new(false));
        let future: Pin<Box<dyn Future<Output = ()> + Send>> = Box::pin(async move { $body });

        let handler = Arc::new(InlineOnceHandler {
            cancel_flag: cancel_flag.clone(),
            future: Mutex::new(Some(future)),
        });

        $server
            .task_scheduler
            .schedule_once($delay_ticks as u64, handler.clone())
            .await;

        ScheduledHandle {
            handler,
            cancel_flag,
        }
    }};
}

#[macro_export]
macro_rules! run_task_timer {
    ($server:expr, $interval_ticks:expr, |$handle_ident:ident| $body:expr) => {{
        use async_trait::async_trait;
        use std::future::Future;
        use std::pin::Pin;
        use std::sync::{
            Arc,
            atomic::{AtomicBool, Ordering},
        };
        use $crate::plugin::api::task::{RepeatingHandle, TaskHandler};

        let cancel_flag = Arc::new(AtomicBool::new(false));
        let handle = RepeatingHandle::new(cancel_flag.clone());
        let handle_arc = Arc::new(handle);

        struct TimerHandler {
            cancel_flag: Arc<AtomicBool>,
            closure: Arc<dyn Fn() -> Pin<Box<dyn Future<Output = ()> + Send>> + Send + Sync>,
        }

        #[async_trait]
        impl TaskHandler for TimerHandler {
            async fn run(&self) {
                if self.cancel_flag.load(Ordering::Relaxed) {
                    return;
                }

                let fut = (self.closure)();
                fut.await;
            }

            async fn cancel(&self) {
                self.cancel_flag.store(true, Ordering::Relaxed);
            }
        }

        let closure_handle = handle_arc.clone();
        let closure = Arc::new(move || {
            let $handle_ident = closure_handle.clone();
            let fut = $body;
            Box::pin(fut) as Pin<Box<dyn Future<Output = ()> + Send>>
        });

        let handler = Arc::new(TimerHandler {
            cancel_flag: cancel_flag.clone(),
            closure,
        });

        $server
            .task_scheduler
            .schedule_repeating($interval_ticks, handler.clone())
            .await;

        handle_arc
    }};
}
