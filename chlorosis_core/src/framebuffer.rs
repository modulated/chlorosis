//! Frame handoff between the emulation thread and the frontend.
//!
//! The two threads run on unrelated clocks: the emulator produces a frame every
//! 16.74 ms of emulated time, the frontend presents whenever the compositor
//! lets it. Passing frames down a queue couples them back together - if the
//! frontend falls behind, an unbounded queue grows without limit and every
//! frame it later shows is already stale, which is exactly the lag this split
//! is meant to remove.
//!
//! So frames are passed through a two slot swap instead. Publishing overwrites
//! whatever the frontend has not collected yet, and the displaced buffer is
//! handed back to be drawn into again. Latency is therefore bounded at one
//! frame no matter how far the two rates drift, and steady state runs without
//! allocating.

use std::sync::{Arc, Mutex, MutexGuard, PoisonError};

pub const SCREEN_WIDTH: usize = 160;
pub const SCREEN_HEIGHT: usize = 144;
pub const FRAME_LEN: usize = SCREEN_WIDTH * SCREEN_HEIGHT;

/// A single completed screen, as 0x00RRGGBB pixels.
pub type Frame = Box<[u32; FRAME_LEN]>;

#[must_use]
pub fn blank_frame() -> Frame {
    Box::new([0; FRAME_LEN])
}

#[derive(Debug, Default)]
struct Slot {
    /// Newest frame the frontend has not taken yet.
    pending: Option<Frame>,
    /// Buffer handed back for reuse.
    spare: Option<Frame>,
}

#[derive(Debug, Default)]
struct Shared(Mutex<Slot>);

impl Shared {
    fn lock(&self) -> MutexGuard<'_, Slot> {
        // Held only for a pointer swap, so contention is negligible. A panic in
        // the emulation thread must not take the frontend down with it, and a
        // half written frame is not observable here: buffers move in and out
        // whole, so recovering from poisoning is safe.
        self.0.lock().unwrap_or_else(PoisonError::into_inner)
    }
}

/// Emulation thread end of the frame swap.
#[derive(Debug, Clone)]
pub struct FrameProducer(Arc<Shared>);

/// Frontend end of the frame swap.
#[derive(Debug, Clone)]
pub struct FrameConsumer(Arc<Shared>);

#[must_use]
pub fn frame_channel() -> (FrameProducer, FrameConsumer) {
    let shared = Arc::new(Shared::default());
    (FrameProducer(Arc::clone(&shared)), FrameConsumer(shared))
}

impl FrameProducer {
    /// Buffer to write the next frame into, recycled where the frontend has
    /// given one back and freshly allocated otherwise.
    #[must_use]
    pub fn acquire(&self) -> Frame {
        self.0.lock().spare.take().unwrap_or_else(blank_frame)
    }

    /// Make `frame` the one the frontend picks up next. Any frame it has not
    /// collected is dropped - showing the newest frame beats showing every
    /// frame late.
    pub fn publish(&self, frame: Frame) {
        let mut slot = self.0.lock();
        let displaced = slot.pending.replace(frame);
        slot.spare = displaced;
    }
}

impl FrameConsumer {
    /// The newest published frame, or `None` if nothing new has arrived. Never
    /// blocks: the frontend redraws its previous frame instead of waiting.
    #[must_use]
    pub fn take(&self) -> Option<Frame> {
        self.0.lock().pending.take()
    }

    /// Hand a finished buffer back to the emulator to be drawn into again.
    pub fn recycle(&self, frame: Frame) {
        let mut slot = self.0.lock();
        if slot.spare.is_none() {
            slot.spare = Some(frame);
        }
    }
}

#[cfg(test)]
mod tests {
    use super::{frame_channel, FRAME_LEN};

    #[test]
    fn take_returns_the_newest_frame_only() {
        let (producer, consumer) = frame_channel();

        let mut first = producer.acquire();
        first[0] = 1;
        producer.publish(first);

        let mut second = producer.acquire();
        second[0] = 2;
        producer.publish(second);

        assert_eq!(consumer.take().expect("a frame is pending")[0], 2);
        assert!(consumer.take().is_none());
    }

    #[test]
    fn buffers_are_recycled_rather_than_reallocated() {
        let (producer, consumer) = frame_channel();

        let mut frame = producer.acquire();
        frame[FRAME_LEN - 1] = 0xABCD;
        let published = frame.as_ptr();
        producer.publish(frame);

        let taken = consumer.take().expect("a frame is pending");
        assert_eq!(taken.as_ptr(), published);
        consumer.recycle(taken);

        assert_eq!(producer.acquire().as_ptr(), published);
    }

    #[test]
    fn overwritten_frames_become_spares() {
        let (producer, consumer) = frame_channel();

        let stale = producer.acquire();
        let stale_ptr = stale.as_ptr();
        producer.publish(stale);
        producer.publish(producer.acquire());

        // The frame the frontend never collected is reused, not leaked.
        assert_eq!(producer.acquire().as_ptr(), stale_ptr);
        assert!(consumer.take().is_some());
    }
}
