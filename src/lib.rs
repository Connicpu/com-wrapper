extern crate winapi;
extern crate wio;

use winapi::Interface;
use wio::com::ComPtr;

pub trait ComWrapper {
    /// The raw interface type from `winapi`
    type Interface: Interface;

    /// Gets a raw pointer to the interface. Does not increment the reference count
    unsafe fn get_raw(&self) -> *mut Self::Interface;

    /// Consumes the wrapper without affecting the reference count
    unsafe fn into_raw(self) -> *mut Self::Interface;

    /// Creates a wrapper from the raw pointer. Takes ownership of the pointer for
    /// reference counting purposes.
    unsafe fn from_raw(raw: *mut Self::Interface) -> Self;

    /// Creates a wrapper taking ownership of a ComPtr.
    unsafe fn from_ptr(ptr: ComPtr<Self::Interface>) -> Self;

    /// Unwraps the wrapper into a ComPtr.
    unsafe fn into_ptr(self) -> ComPtr<Self::Interface>;
}
