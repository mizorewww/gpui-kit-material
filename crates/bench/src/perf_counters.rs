//! Main-thread counters for instruction-based A/B benchmarks: retired
//! instructions and cycles (the counter upstream `gpui_perf` reads), CPU time
//! and heap allocations. They do not depend on machine load, which wall-clock
//! numbers do. Native macOS reads the real counters; elsewhere they read 0.
use std::{
    alloc::{GlobalAlloc, Layout, System},
    cell::Cell,
};

thread_local! {
    static ALLOCATIONS: Cell<u64> = const { Cell::new(0) };
    static ALLOCATED_BYTES: Cell<u64> = const { Cell::new(0) };
}

/// The system allocator, counting the calling thread's allocations. Install it
/// as the `#[global_allocator]` of a benchmark binary.
pub struct CountingAllocator;

// SAFETY: every call forwards to `System`; counting touches only a const-init
// thread local, which neither allocates nor unwinds.
unsafe impl GlobalAlloc for CountingAllocator {
    unsafe fn alloc(&self, layout: Layout) -> *mut u8 {
        count(layout.size());
        unsafe { System.alloc(layout) }
    }
    unsafe fn dealloc(&self, ptr: *mut u8, layout: Layout) {
        unsafe { System.dealloc(ptr, layout) }
    }
    unsafe fn alloc_zeroed(&self, layout: Layout) -> *mut u8 {
        count(layout.size());
        unsafe { System.alloc_zeroed(layout) }
    }
    unsafe fn realloc(&self, ptr: *mut u8, layout: Layout, new_size: usize) -> *mut u8 {
        count(new_size);
        unsafe { System.realloc(ptr, layout, new_size) }
    }
}

fn count(bytes: usize) {
    let _ = ALLOCATIONS.try_with(|count| count.set(count.get() + 1));
    let _ = ALLOCATED_BYTES.try_with(|total| total.set(total.get() + bytes as u64));
}

/// One reading of the calling thread's counters.
#[derive(Clone, Copy, Debug, Default)]
pub struct Counters {
    pub instructions: u64,
    pub cycles: u64,
    pub cpu_ns: u64,
    pub allocations: u64,
    pub allocated_bytes: u64,
}

impl Counters {
    pub fn now() -> Self {
        let (instructions, cycles) = thread_counts();
        Self {
            instructions,
            cycles,
            cpu_ns: thread_cpu_ns(),
            allocations: ALLOCATIONS.with(Cell::get),
            allocated_bytes: ALLOCATED_BYTES.with(Cell::get),
        }
    }

    /// What happened since `earlier`.
    pub fn since(self, earlier: Self) -> Self {
        Self {
            instructions: self.instructions.saturating_sub(earlier.instructions),
            cycles: self.cycles.saturating_sub(earlier.cycles),
            cpu_ns: self.cpu_ns.saturating_sub(earlier.cpu_ns),
            allocations: self.allocations.saturating_sub(earlier.allocations),
            allocated_bytes: self.allocated_bytes.saturating_sub(earlier.allocated_bytes),
        }
    }

    /// `[instructions, cycles, cpu_ns, allocations, allocated_bytes]`, the
    /// order the summarizer reads raw frames in.
    pub fn row(self) -> [u64; 5] {
        [
            self.instructions,
            self.cycles,
            self.cpu_ns,
            self.allocations,
            self.allocated_bytes,
        ]
    }
}

#[cfg(target_os = "macos")]
fn thread_counts() -> (u64, u64) {
    unsafe extern "C" {
        fn thread_selfcounts(kind: i32, buffer: *mut u64, size: usize) -> i32;
    }
    const THSC_CPI: i32 = 1;
    let mut counts = [0u64; 2];
    // SAFETY: the buffer holds the two counters this kind writes.
    let status = unsafe { thread_selfcounts(THSC_CPI, counts.as_mut_ptr(), 16) };
    if status == 0 {
        (counts[0], counts[1])
    } else {
        (0, 0)
    }
}

#[cfg(not(target_os = "macos"))]
fn thread_counts() -> (u64, u64) {
    (0, 0)
}

#[cfg(target_os = "macos")]
fn thread_cpu_ns() -> u64 {
    unsafe extern "C" {
        fn clock_gettime_nsec_np(clock_id: u32) -> u64;
    }
    const CLOCK_THREAD_CPUTIME_ID: u32 = 16;
    // SAFETY: a pure query of the calling thread's CPU clock.
    unsafe { clock_gettime_nsec_np(CLOCK_THREAD_CPUTIME_ID) }
}

#[cfg(not(target_os = "macos"))]
fn thread_cpu_ns() -> u64 {
    0
}
