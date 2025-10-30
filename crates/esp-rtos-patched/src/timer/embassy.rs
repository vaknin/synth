use core::task::Waker;

use embassy_time_queue_utils::Queue;
use esp_sync::NonReentrantMutex;

use crate::SCHEDULER;

pub(super) struct TimerQueueInner {
    queue: Queue,
    pub next_wakeup: u64,
}

impl TimerQueueInner {
    const fn new() -> Self {
        Self {
            queue: Queue::new(),
            next_wakeup: u64::MAX,
        }
    }

    pub(crate) fn handle_alarm(&mut self, now: u64) {
        if now < self.next_wakeup {
            return;
        }

        self.next_wakeup = self.queue.next_expiration(now);
    }

    fn schedule_wake(&mut self, at: u64, waker: &Waker) -> bool {
        if self.queue.schedule_wake(at, waker) {
            self.next_wakeup = self.next_wakeup.min(at);
            true
        } else {
            false
        }
    }
}

pub(crate) struct TimerQueue {
    inner: NonReentrantMutex<TimerQueueInner>,
}

impl TimerQueue {
    pub(crate) const fn new() -> Self {
        Self {
            inner: NonReentrantMutex::new(TimerQueueInner::new()),
        }
    }

    pub(crate) fn handle_alarm(&self, now: u64) {
        self.inner.with(|inner| inner.handle_alarm(now))
    }

    pub(crate) fn next_wakeup(&self) -> u64 {
        self.inner.with(|inner| inner.next_wakeup)
    }
}

impl embassy_time_driver::Driver for TimerQueue {
    #[inline]
    fn now(&self) -> u64 {
        crate::now()
    }

    #[inline]
    fn schedule_wake(&self, at: u64, waker: &Waker) {
        // Note that we don't put the thread to sleep here, as other embassy tasks may be
        // ready. The thread will go to sleep when it can.

        // Break circular dependency: capture next_wakeup inside lock, pass as parameter
        // to arm_next_wakeup() (which calls next_wakeup()) to avoid reentering this lock.
        // Note: Small TOCTOU race where next_wakeup may become stale before arm_next_wakeup()
        // is called. This is acceptable; the system tolerates waking slightly late as the
        // timer is re-armed on every tick.
        let needs_rearm = self.inner.with(|inner| {
            let changed = inner.schedule_wake(at, waker);
            if changed {
                Some(inner.next_wakeup)
            } else {
                None
            }
        });

        // Rearm outside the lock (lock released above)
        if let Some(embassy_next_wakeup) = needs_rearm {
            SCHEDULER.with(|scheduler| {
                unwrap!(scheduler.time_driver.as_mut())
                    .arm_next_wakeup(crate::now(), embassy_next_wakeup);
            });
        }
    }
}
