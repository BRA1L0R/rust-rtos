use alloc_cortex_m::CortexMHeap;

#[global_allocator]
static ALLOCATOR: CortexMHeap = CortexMHeap::empty();

/// # Safety
/// safety of [`CortexMHeap::init`]
pub unsafe fn init_allocator(start: *const u32, size: usize) {
    ALLOCATOR.init(start as _, size)
}

/// (free, used)
pub fn free() -> (usize, usize) {
    (ALLOCATOR.free(), ALLOCATOR.used())
}
