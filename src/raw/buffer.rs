use std::sync::atomic::{AtomicUint, AtomicPtr, Ordering};
use std::num::Int;
use std::mem;
use alloc::heap;

#[unsafe_no_drop_flag]
pub struct Buffer<T> {
    capacity: AtomicUint,
    buffer: AtomicPtr<T>
}

impl<T> Buffer<T> {
    #[inline]
    pub fn allocate(capacity: uint) -> Buffer<T> {
        Buffer {
            capacity: AtomicUint::new(0),
            buffer: AtomicPtr::new(unsafe { allocate_or_empty(capacity) })
        }
    }

    #[inline]
    pub fn empty() -> Buffer<T> {
        Buffer::allocate(0)
    }

    #[inline]
    pub unsafe fn get(&self, index: uint, ordering: Ordering) -> *const T {
        mem::transmute(self.buffer.load(ordering).offset(index as int))
    }

    #[inline]
    pub unsafe fn get_mut(&self, index: uint, ordering: Ordering) -> *mut T {
        mem::transmute(self.buffer.load(ordering).offset(index as int))
    }

    #[inline]
    pub unsafe fn set(&mut self, index: uint, data: T, ordering: Ordering) {
        *self.get_mut(index, ordering) = data;
    }

    /// UB if:
    ///   - capacity == 0
    #[inline]
    pub unsafe fn reallocate(&mut self, capacity: uint, ordering: Ordering) {
        debug_assert!(capacity != 0);

        if mem::size_of::<T>() == 0 { return }

        let old_capacity = self.capacity.load(ordering);

        let ptr = if old_capacity == 0 {
            allocate(capacity)
        } else {
            reallocate(self.buffer.load(ordering), old_capacity, capacity)
        };

        self.buffer.store(ptr, ordering);
    }

    /// Prior to a drop can be used to deallocate with a non-SeqCst
    /// memory ordering.
    #[inline]
    pub unsafe fn deallocate(&mut self, ordering: Ordering) {
        if mem::size_of::<T>() == 0 { return }

        let capacity = self.capacity.swap(0, ordering);
        let buffer = self.buffer.swap(empty(), ordering);
        deallocate(buffer, capacity);
    }
}

#[unsafe_destructor]
impl<T> Drop for Buffer<T> {
    fn drop(&mut self) {
        if mem::size_of::<T>() == 0 { return }

        if self.capacity.load(Ordering::SeqCst) == 0 { return }

        unsafe { self.deallocate(Ordering::SeqCst) }
    }
}

#[inline]
unsafe fn empty<T>() -> *mut T { 1u as *mut T }

#[inline]
unsafe fn allocate_or_empty<T>(capacity: uint) -> *mut T {
    if mem::size_of::<T>() == 0 || capacity == 0 {
        empty()
    } else {
        allocate(capacity)
    }
}

/// UB if:
///   - capacity == 0
///   - size_of::<T> == 0
#[inline]
unsafe fn allocate<T>(capacity: uint) -> *mut T {
    let size_of = mem::size_of::<T>();
    let alignment = mem::align_of::<T>();

    debug_assert!(size_of != 0);
    debug_assert!(capacity != 0);

    let size = allocation_size::<T>(size_of);
    let ptr = heap::allocate(size, alignment);
    if ptr.is_null() { ::alloc::oom() }

    ptr as *mut T
}

/// UB if:
///   - new_capacity == 0
///   - size_of::<T> == 0
///   - old is not allocated by the heap allocator
///   - old_capacity is not the capacity of old
#[inline]
unsafe fn reallocate<T>(old: *mut T, old_capacity: uint, new_capacity: uint) -> *mut T {
    let size_of = mem::size_of::<T>();
    let alignment = mem::align_of::<T>();

    debug_assert!(size_of != 0);
    debug_assert!(old_capacity != 0);
    debug_assert!(new_capacity != 0);

    let ptr = heap::reallocate(old as *mut u8, allocation_size::<T>(old_capacity),
                               allocation_size::<T>(new_capacity), alignment);
    if ptr.is_null() { ::alloc::oom() }

    ptr as *mut T
}

/// UB if:
///   - capacity == 0
///   - size_of::<T> == 0
///   - old is not allocated by the heap allocator
///   - capacity is not the capacity of old
#[inline]
unsafe fn deallocate<T>(old: *mut T, capacity: uint) {
    let size_of = mem::size_of::<T>();
    let alignment = mem::align_of::<T>();

    debug_assert!(size_of != 0);
    debug_assert!(capacity != 0);

    let size = allocation_size::<T>(size_of);
    heap::deallocate(old as *mut u8, size, alignment)
}

/// Capacity should not == 0 or this will give not-usable results
/// same for size_of::<T>
#[inline]
fn allocation_size<T>(capacity: uint) -> uint {
    debug_assert!(capacity != 0);
    debug_assert!(mem::size_of::<T>() != 0);

    capacity.checked_mul(mem::size_of::<T>()).expect("capacity overflow")
}

