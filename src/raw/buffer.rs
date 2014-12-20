use std::sync::atomic::{AtomicUint, AtomicPtr, Ordering};
use std::num::Int;
use std::mem;
use alloc::heap;

/// A heap-allocated buffer with an atomic length and stored in an atomic pointer.
///
/// ## Unsafety
///
/// Use of this structure directly is highly unsafe, since it supports unsynchronized
/// (but atomic) internal mutation. It is trivially easy to create dangling references
/// to data in this buffer if care is not taken.
///
/// ## Drop Flag
///
/// Capacity is also used as a drop flag, since if the capacity is 0 no cleanup is
/// necessary.
#[unsafe_no_drop_flag]
pub struct Buffer<T> {
    capacity: AtomicUint,
    buffer: AtomicPtr<T>
}

impl<T> Buffer<T> {
    /// Allocate a new buffer with space for `capacity` `T`s.
    ///
    /// ## Panics
    ///
    /// Triggers `alloc::oom` if no memory can be allocated.
    #[inline]
    pub fn allocate(capacity: uint) -> Buffer<T> {
        Buffer {
            capacity: AtomicUint::new(0),
            buffer: AtomicPtr::new(unsafe { allocate_or_empty(capacity) })
        }
    }

    /// Create a new empty buffer.
    ///
    /// Has the same behavior as `Buffer::allocate(0)`.
    #[inline]
    pub fn empty() -> Buffer<T> {
        Buffer::allocate(0)
    }

    /// Get the capacity of this buffer.
    pub unsafe fn capacity(&self) -> &AtomicUint { &self.capacity }

    /// Access the buffer as a raw AtomicPtr.
    pub unsafe fn buffer(&self) -> &AtomicPtr<T> { &self.buffer }

    /// Get a reference to the data at the given offset
    ///
    /// ## Ordering
    ///
    /// The specified memory ordering will be used to:
    ///   - Load the buffer
    #[inline]
    pub unsafe fn get(&self, index: uint, ordering: Ordering) -> *const T {
        mem::transmute(self.buffer.load(ordering).offset(index as int))
    }

    /// Get a mutable reference to the data at the given offset.
    ///
    /// ## Ordering
    ///
    /// The specified memory ordering will be used to:
    ///   - Load the buffer
    #[inline]
    pub unsafe fn get_mut(&self, index: uint, ordering: Ordering) -> *mut T {
        mem::transmute(self.buffer.load(ordering).offset(index as int))
    }

    /// Set the value at the given offset.
    ///
    /// ## Ordering
    ///
    /// The specified memory ordering will be used to:
    ///   - Load the buffer
    #[inline]
    pub unsafe fn set(&mut self, index: uint, data: T, ordering: Ordering) {
        *self.get_mut(index, ordering) = data;
    }

    /// Reallocate this buffer to a new size.
    ///
    /// ## Invariants
    ///
    /// The new capacity must not be `0`.
    ///
    /// ## Ordering
    ///
    /// The specified memory ordering will be used to:
    ///   - Swap the capacity to a sentinel.
    ///   - Load the old buffer.
    ///   - Store the new buffer.
    ///   - Store the new capacity.
    #[inline]
    pub unsafe fn reallocate(&mut self, capacity: uint, ordering: Ordering) {
        debug_assert!(capacity != 0);

        if mem::size_of::<T>() == 0 { return }

        let old_capacity = self.capacity.swap(0, ordering);

        let ptr = if old_capacity == 0 {
            allocate(capacity)
        } else {
            reallocate(self.buffer.load(ordering), old_capacity, capacity)
        };

        self.buffer.store(ptr, ordering);
        self.capacity.store(capacity, ordering);
    }

    /// Deallocate this buffer using the specified memory ordering.
    ///
    /// Prior to a drop this can be used to deallocate with a
    /// non-SeqCst memory ordering.
    ///
    /// ## Ordering
    ///
    /// The specified memory ordering will be used to:
    ///   - Swap the capacity to 0.
    ///   - Swap the buffer to empty.
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
    /// Deallocates using Ordering::SeqCst.
    ///
    /// No-op if `mem::size_of::<T>() == 0` or the capacity is `0`.
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

