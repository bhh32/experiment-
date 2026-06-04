//! True (non-async) parallelism helpers built on `std::thread`.

use std::sync::mpsc;
use std::sync::{Arc, Mutex};
use std::time::Duration;

use indicatif::{ProgressBar, ProgressStyle};

/// Apply `f` to every item across up to `max_workers` OS threads, preserving
/// input order in the returned vector.
///
/// This uses a scoped thread pool pulling from a shared work queue over an
/// `mpsc` channel. It performs no async I/O and blocks until all work is done.
pub fn parallel_map<T, R, F>(items: Vec<T>, max_workers: usize, f: F) -> Vec<R>
where
    T: Send,
    R: Send,
    F: Fn(T) -> R + Send + Sync,
{
    let len = items.len();
    if len == 0 {
        return Vec::new();
    }
    let workers = max_workers.clamp(1, len);

    // Shared queue of (index, item); results land in a pre-sized slot vector.
    let (tx, rx) = mpsc::channel::<(usize, T)>();
    for (i, item) in items.into_iter().enumerate() {
        // Send cannot fail: the receiver lives for the whole scope below.
        tx.send((i, item)).expect("send work item");
    }
    drop(tx);

    let rx = Arc::new(Mutex::new(rx));
    let mut results: Vec<Option<R>> = (0..len).map(|_| None).collect();
    let results_slots: Arc<Mutex<&mut Vec<Option<R>>>> = Arc::new(Mutex::new(&mut results));

    std::thread::scope(|scope| {
        for _ in 0..workers {
            let rx = Arc::clone(&rx);
            let slots = Arc::clone(&results_slots);
            let f = &f;
            scope.spawn(move || loop {
                // Lock only to pull the next item, then release before working.
                let next = {
                    let guard = rx.lock().expect("lock receiver");
                    guard.recv()
                };
                let Ok((idx, item)) = next else {
                    break;
                };
                let result = f(item);
                let mut out = slots.lock().expect("lock results");
                out[idx] = Some(result);
            });
        }
    });

    results.into_iter().map(|r| r.expect("all slots filled")).collect()
}

/// Drive a poll loop on the main thread with an `indicatif` spinner, returning
/// the first `Some(T)` produced by `poll`.
///
/// `poll` is invoked every `interval`; the spinner ticks between polls.
pub fn poll_with_progress<F, T>(label: &str, interval: Duration, mut poll: F) -> T
where
    F: FnMut() -> Option<T>,
{
    let bar = ProgressBar::new_spinner();
    bar.set_style(
        ProgressStyle::with_template("{spinner} {msg}")
            .unwrap_or_else(|_| ProgressStyle::default_spinner()),
    );
    bar.set_message(label.to_string());

    loop {
        if let Some(value) = poll() {
            bar.finish_and_clear();
            return value;
        }
        // Tick the spinner across the interval for visible motion.
        let tick = Duration::from_millis(100);
        let mut elapsed = Duration::ZERO;
        while elapsed < interval {
            bar.tick();
            std::thread::sleep(tick);
            elapsed += tick;
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::sync::atomic::{AtomicUsize, Ordering};

    #[test]
    fn parallel_map_squares() {
        let input: Vec<u64> = (0..100).collect();
        let out = parallel_map(input.clone(), 8, |x| x * x);
        assert_eq!(out.len(), 100);
        for (i, v) in out.iter().enumerate() {
            assert_eq!(*v, (i as u64) * (i as u64));
        }
    }

    #[test]
    fn parallel_map_empty() {
        let out = parallel_map(Vec::<u64>::new(), 4, |x| x + 1);
        assert!(out.is_empty());
    }

    #[test]
    fn parallel_map_runs_all() {
        let counter = AtomicUsize::new(0);
        let out = parallel_map((0..50).collect(), 4, |x| {
            counter.fetch_add(1, Ordering::SeqCst);
            x
        });
        assert_eq!(out.len(), 50);
        assert_eq!(counter.load(Ordering::SeqCst), 50);
    }

    #[test]
    fn poll_returns_first_some() {
        let mut n = 0;
        let v = poll_with_progress("waiting", Duration::from_millis(1), || {
            n += 1;
            if n >= 3 {
                Some(n)
            } else {
                None
            }
        });
        assert_eq!(v, 3);
    }
}
