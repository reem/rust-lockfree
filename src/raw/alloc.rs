use alloc::heap;
use std::mem;
use std::num::Int;

#[inline]
pub unsafe fn empty<T>() -> *mut T { 1u as *mut T }

/// UB if:
///   - capacity == 0
///   - size_of::<T> == 0
#[inline]
pub unsafe fn allocate<T>(capacity: uint) -> *mut T {
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
pub unsafe fn reallocate<T>(old: *mut T, old_capacity: uint, new_capacity: uint) -> *mut T {
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
pub unsafe fn deallocate<T>(old: *mut T, capacity: uint) {
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

