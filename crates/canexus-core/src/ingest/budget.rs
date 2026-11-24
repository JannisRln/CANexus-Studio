//! Shared accounting for live batch allocations.

use std::sync::{
    atomic::{AtomicUsize, Ordering::Relaxed},
    Arc,
};

/// A byte limit shared by producers and every batch they hand out.
///
/// Clone this handle when constructing producers that must share one limit.
#[derive(Clone, Debug)]
pub struct BatchBudget(Arc<Usage>);

#[derive(Debug)]
struct Usage {
    limit: usize,
    live: AtomicUsize,
    peak: AtomicUsize,
}

impl BatchBudget {
    /// Creates a budget for staged and published batch storage.
    #[must_use]
    pub fn new(limit_bytes: usize) -> Self {
        Self(Arc::new(Usage {
            limit: limit_bytes,
            live: AtomicUsize::new(0),
            peak: AtomicUsize::new(0),
        }))
    }

    /// Maximum charge across all producers sharing this budget.
    #[must_use]
    pub fn limit_bytes(&self) -> usize {
        self.0.limit
    }

    /// Current charge, including storage pinned outside subscriber queues.
    #[must_use]
    pub fn live_bytes(&self) -> usize {
        self.0.live.load(Relaxed)
    }

    /// Highest charge reached since creation.
    #[must_use]
    pub fn peak_bytes(&self) -> usize {
        self.0.peak.load(Relaxed)
    }

    pub(super) fn reserve(&self, bytes: usize) -> Option<BatchLease> {
        let previous = self
            .0
            .live
            .try_update(Relaxed, Relaxed, |live| {
                live.checked_add(bytes).filter(|&next| next <= self.0.limit)
            })
            .ok()?;
        self.0.peak.fetch_max(previous + bytes, Relaxed);
        Some(BatchLease {
            budget: self.clone(),
            bytes,
        })
    }
}

/// Owns one charge, moved from the producer to the shared batch.
#[derive(Debug)]
pub(super) struct BatchLease {
    budget: BatchBudget,
    bytes: usize,
}

impl Drop for BatchLease {
    fn drop(&mut self) {
        self.budget.0.live.fetch_sub(self.bytes, Relaxed);
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::{sync::Barrier, thread};

    #[test]
    fn concurrent_reservations_cannot_overcommit() {
        let budget = BatchBudget::new(64);
        let ready = Barrier::new(2);
        let attempted = Barrier::new(2);

        let successes = thread::scope(|scope| {
            let reserve = || {
                ready.wait();
                let lease = budget.reserve(64);
                attempted.wait();
                lease.is_some()
            };
            let other = scope.spawn(reserve);
            usize::from(reserve()) + usize::from(other.join().unwrap())
        });

        assert_eq!(successes, 1);
        assert_eq!(budget.peak_bytes(), 64);
        assert_eq!(budget.live_bytes(), 0);
    }
}
