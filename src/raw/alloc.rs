use alloc::heap;
use std::mem;
use std::num::Int;

/// "Allocate" a special allocation of zero size.
#[inline]
pub unsafe fn empty<T>() -> *mut T { 1u as *mut T }

/// Allocate space for `capacity` `T`s
///
/// ## Panics
///
/// Will call `alloc::oom` if allocation fails.
///
/// ## Invariants
///   - `capacity` is non-zero.
///   - `T` is not a zero-sized-type.
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


/// Reallocate `old` to a new size, so it can hold `new_capacity` `T`s
///
/// ## Panics
///
/// Will call `alloc::oom` if reallocation fails.
///
/// ## Invariants
///   - `old_capacity` is non-zero.
///   - `new_capacity` is non-zero.
///   - `T` is not a zero-sized-type.
///   - `old` is the appropriate size for `old_capacity` `T`s size and was allocated by
///     the heap allocator.
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

/// Deallocates `old`
///
/// ## Invariants
///   - `capacity` is non-zero.
///   - `T` is not a zero-sized-type.
///   - `old` is the appropriate size for `capacity` `T`s size and was allocated by
///     the heap allocator.
#[inline]
pub unsafe fn deallocate<T>(old: *mut T, capacity: uint) {
    let size_of = mem::size_of::<T>();
    let alignment = mem::align_of::<T>();

    debug_assert!(size_of != 0);
    debug_assert!(capacity != 0);

    let size = allocation_size::<T>(size_of);
    heap::deallocate(old as *mut u8, size, alignment)
}

/// Gets the appropriate size for an allocation of `capacity` `T`s, checking for overflow.
///
/// ## Invariants
///   - `capacity` is non-zero.
///   - `T` is not a zero-sized-type.
#[inline]
fn allocation_size<T>(capacity: uint) -> uint {
    debug_assert!(capacity != 0);
    debug_assert!(mem::size_of::<T>() != 0);

    capacity.checked_mul(mem::size_of::<T>()).expect("capacity overflow")
}

