//! Counts the heap bytes each thread holds, so a parse can stop once it holds
//! more memory than its budget allows.

#![deny(unsafe_op_in_unsafe_fn)]

use std::alloc::{GlobalAlloc, Layout, System};
use std::cell::Cell;

/// The system allocator, counting the bytes each thread allocates and frees.
pub struct CountingAllocator;

thread_local! {
    // Bytes this thread allocated minus bytes it freed. Memory freed on a
    // different thread than the one that allocated it skews both threads'
    // counts, so only the change over work done on one thread is meaningful.
    static HELD: Cell<isize> = const { Cell::new(0) };
    // The most HELD has been since the last `start_peak`.
    static PEAK: Cell<isize> = const { Cell::new(0) };
}

/// Bytes this thread has allocated minus the bytes it has freed. The change
/// between two readings is the memory the work in between still holds.
pub fn held() -> isize {
    HELD.try_with(Cell::get).unwrap_or(0)
}

/// Starts tracking the most this thread holds, and returns what it holds now.
pub fn start_peak() -> isize {
    let held = held();
    let _ = PEAK.try_with(|peak| peak.set(held));
    held
}

/// The most this thread has held since the last `start_peak`.
pub fn peak() -> isize {
    PEAK.try_with(Cell::get).unwrap_or(0)
}

fn count(bytes: isize) {
    // An allocation can happen after this thread's locals are torn down, and
    // an allocator must not panic.
    let _ = HELD.try_with(|held| {
        let now = held.get().wrapping_add(bytes);
        held.set(now);
        let _ = PEAK.try_with(|peak| peak.set(peak.get().max(now)));
    });
}

// `Layout` sizes never exceed `isize::MAX`, and neither does `realloc`'s
// `new_size`, so the casts below are lossless.
unsafe impl GlobalAlloc for CountingAllocator {
    unsafe fn alloc(&self, layout: Layout) -> *mut u8 {
        // SAFETY: the caller upholds `alloc`'s contract, which `System` shares.
        let ptr = unsafe { System.alloc(layout) };
        if !ptr.is_null() {
            count(layout.size() as isize);
        }
        ptr
    }

    unsafe fn alloc_zeroed(&self, layout: Layout) -> *mut u8 {
        // SAFETY: the caller upholds `alloc_zeroed`'s contract, which `System` shares.
        let ptr = unsafe { System.alloc_zeroed(layout) };
        if !ptr.is_null() {
            count(layout.size() as isize);
        }
        ptr
    }

    unsafe fn dealloc(&self, ptr: *mut u8, layout: Layout) {
        // SAFETY: every block this allocator hands out comes from `System`, and
        // the caller passes the layout it was allocated with.
        unsafe { System.dealloc(ptr, layout) };
        count(-(layout.size() as isize));
    }

    unsafe fn realloc(&self, ptr: *mut u8, layout: Layout, new_size: usize) -> *mut u8 {
        // SAFETY: as in `dealloc`, `ptr` came from `System` with `layout`, and the
        // caller upholds `realloc`'s contract for `new_size`.
        let new_ptr = unsafe { System.realloc(ptr, layout, new_size) };
        if !new_ptr.is_null() {
            count(new_size as isize - layout.size() as isize);
        }
        new_ptr
    }
}

#[cfg(test)]
mod tests {
    use super::{held, peak, start_peak};
    // The optimizer may drop or shrink allocations it can see aren't used,
    // so the tests hide their buffers behind `black_box`.
    use std::hint::black_box;

    const MIB: isize = 1 << 20;

    #[test]
    fn counts_what_this_thread_allocates_and_frees() {
        let before = held();
        let mut buffer = black_box(vec![0u8; MIB as usize]);
        assert!(held() - before >= MIB);

        buffer.resize(4 * MIB as usize, 0);
        black_box(&buffer);
        assert!(held() - before >= 4 * MIB);

        drop(buffer);
        assert!(held() - before < MIB);
    }

    #[test]
    fn peak_keeps_the_most_held_after_a_free() {
        let start = start_peak();
        drop(black_box(vec![0u8; 4 * MIB as usize]));
        assert!(held() - start < MIB);
        assert!(peak() - start >= 4 * MIB);
    }

    #[test]
    fn ignores_what_other_threads_allocate() {
        let before = held();
        let kept = std::thread::spawn(|| black_box(vec![0u8; 4 * MIB as usize]))
            .join()
            .unwrap();
        assert!(held() - before < MIB);
        assert_eq!(kept.len(), 4 * MIB as usize);
    }
}
