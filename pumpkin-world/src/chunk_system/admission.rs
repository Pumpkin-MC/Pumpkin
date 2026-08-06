use std::sync::{
    Arc,
    atomic::{AtomicUsize, Ordering},
};

const GENERATION_QUEUE_SLACK: usize = 2;

/// Shared Rayon pool and admission budget used by every level on a server.
pub struct GenerationRuntime {
    pool: Arc<rayon::ThreadPool>,
    admission: Arc<GenerationAdmission>,
}

impl GenerationRuntime {
    /// Builds a runtime with a small amount of queued work beyond the worker count.
    pub fn new(worker_count: usize) -> Result<Arc<Self>, rayon::ThreadPoolBuildError> {
        let worker_count = worker_count.max(1);
        let pool = Arc::new(
            rayon::ThreadPoolBuilder::new()
                .num_threads(worker_count)
                .thread_name(|i| format!("Gen-Pool-{i}"))
                .build()?,
        );
        let limit = worker_count.saturating_add(GENERATION_QUEUE_SLACK);
        Ok(Arc::new(Self {
            pool,
            admission: Arc::new(GenerationAdmission::new(limit)),
        }))
    }

    #[must_use]
    pub const fn pool(&self) -> &Arc<rayon::ThreadPool> {
        &self.pool
    }

    #[must_use]
    pub const fn admission(&self) -> &Arc<GenerationAdmission> {
        &self.admission
    }

    #[must_use]
    pub fn worker_count(&self) -> usize {
        self.pool.current_num_threads()
    }

    #[must_use]
    pub fn admission_limit(&self) -> usize {
        self.admission.limit
    }
}

/// A process-wide cap on chunk-generation jobs submitted to Rayon.
pub struct GenerationAdmission {
    limit: usize,
    admitted: AtomicUsize,
}

impl GenerationAdmission {
    #[must_use]
    pub const fn new(limit: usize) -> Self {
        Self {
            limit: if limit == 0 { 1 } else { limit },
            admitted: AtomicUsize::new(0),
        }
    }

    #[must_use]
    pub fn try_acquire(self: &Arc<Self>) -> Option<GenerationPermit> {
        self.admitted
            .fetch_update(Ordering::AcqRel, Ordering::Acquire, |current| {
                (current < self.limit).then_some(current + 1)
            })
            .ok()
            .map(|_| GenerationPermit {
                admission: self.clone(),
            })
    }

    #[must_use]
    pub fn admitted(&self) -> usize {
        self.admitted.load(Ordering::Acquire)
    }

    #[must_use]
    pub const fn limit(&self) -> usize {
        self.limit
    }
}

/// Releases one shared admission slot when the generation result is consumed or dropped.
pub struct GenerationPermit {
    admission: Arc<GenerationAdmission>,
}

impl Drop for GenerationPermit {
    fn drop(&mut self) {
        let released =
            self.admission
                .admitted
                .fetch_update(Ordering::AcqRel, Ordering::Acquire, |current| {
                    current.checked_sub(1)
                });
        debug_assert!(released.is_ok(), "generation admission permit underflow");
    }
}

#[cfg(test)]
mod tests {
    use super::GenerationAdmission;
    use std::sync::Arc;

    #[test]
    fn permits_are_bounded_and_released_on_drop() {
        let admission = Arc::new(GenerationAdmission::new(2));
        let first = admission.try_acquire().unwrap();
        let second = admission.try_acquire().unwrap();
        assert!(admission.try_acquire().is_none());
        assert_eq!(admission.admitted(), 2);

        drop(first);
        let replacement = admission.try_acquire().unwrap();
        assert_eq!(admission.admitted(), 2);
        drop((second, replacement));
        assert_eq!(admission.admitted(), 0);
    }
}
