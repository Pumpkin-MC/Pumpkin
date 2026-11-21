use std::future::Future;
use tokio::runtime::Handle;

/// Runs a synchronous command callback on the server's Tokio runtime.
pub fn block_on_callback<F: Future>(runtime: &Handle, callback: F) -> F::Output {
    if Handle::try_current().is_ok() {
        tokio::task::block_in_place(|| runtime.block_on(callback))
    } else {
        // Simulation workers have no entered runtime; use the server's handle.
        runtime.block_on(callback)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::{thread, time::Duration};
    use tokio::runtime::Builder;

    #[test]
    fn callbacks_run_with_or_without_an_entered_runtime() {
        let runtime = Builder::new_multi_thread()
            .worker_threads(1)
            .enable_all()
            .build()
            .unwrap();
        let handle = runtime.handle().clone();
        let simulation_thread = thread::spawn(move || {
            assert!(Handle::try_current().is_err());
            block_on_callback(&handle, async {
                tokio::time::sleep(Duration::from_millis(1)).await;
                tokio::spawn(async { 42 }).await.unwrap()
            })
        });
        assert_eq!(simulation_thread.join().unwrap(), 42);
        let handle = runtime.handle().clone();
        runtime.block_on(async {
            assert_eq!(
                block_on_callback(&handle, async {
                    tokio::time::sleep(Duration::from_millis(1)).await;
                    tokio::spawn(async { 7 }).await.unwrap()
                }),
                7
            );
        });
    }
}
