use std::hint::spin_loop;
use std::thread;
use std::time::{Duration, Instant};

pub fn wait_frame(next_frame: Instant, spin_time: Duration) {
    if let Some(wait) = next_frame
        .checked_duration_since(Instant::now())
        .and_then(|remaining| remaining.checked_sub(spin_time))
    {
        thread::sleep(wait);
    }

    while Instant::now() < next_frame {
        spin_loop();
    }
}

pub type Tick = u32;

pub trait TickMethods {
    const HALF_RANGE: u32;

    fn zero() -> Self;
    fn next(&mut self) -> Self;

    fn is_after(self, tick: Self) -> bool;
    fn is_before(self, tick: Self) -> bool;
    fn is_after_or_equal(self, tick: Self) -> bool;
    fn is_before_or_equal(self, tick: Self) -> bool;

    fn newest_between(self, tick_1: Self, tick_2: Self) -> Self;
    fn oldest_between(self, tick_1: Self, tick_2: Self) -> Self;
}

impl TickMethods for Tick {
    const HALF_RANGE: u32 = 1 << 31;

    #[inline]
    fn zero() -> Self {
        0
    }

    #[inline]
    fn next(&mut self) -> Self {
        *self = self.wrapping_add(1);
        *self
    }

    #[inline]
    fn is_after(self, tick: Self) -> bool {
        self != tick && self.is_after_or_equal(tick)
    }

    #[inline]
    fn is_before(self, tick: Self) -> bool {
        self != tick && self.is_before_or_equal(tick)
    }

    #[inline]
    fn is_after_or_equal(self, tick: Self) -> bool {
        self.wrapping_sub(tick) < Self::HALF_RANGE
    }

    #[inline]
    fn is_before_or_equal(self, tick: Self) -> bool {
        tick.is_after_or_equal(self)
    }

    #[inline]
    fn newest_between(self, tick_1: Self, tick_2: Self) -> Self {
        if self.wrapping_sub(tick_1) < self.wrapping_sub(tick_2) {
            tick_1
        } else {
            tick_2
        }
    }

    #[inline]
    fn oldest_between(self, tick_1: Self, tick_2: Self) -> Self {
        if self.wrapping_sub(tick_1) > self.wrapping_sub(tick_2) {
            tick_1
        } else {
            tick_2
        }
    }
}
