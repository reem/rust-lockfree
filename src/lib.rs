#![feature(unsafe_destructor)]
#![deny(warnings)]
//#![deny(missing_docs)]

//! Lock-free data concurrent data structures.

extern crate alloc;

/// Lock-free data structures that expose an unsafe interface.
///
/// These are used internally in `lockfree` and exposed here
/// for external use, should they be convenient.
pub mod raw {
    pub mod ringbuf;
    pub mod buffer;
}

