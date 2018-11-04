#[macro_use]
extern crate derive_com_wrapper;
extern crate com_wrapper;
extern crate winapi;
extern crate wio;

use com_wrapper::ComWrapper;

use winapi::um::unknwnbase::IUnknown;
use wio::com::ComPtr;

#[derive(ComWrapper)]
#[com(send, sync)]
#[repr(transparent)]
pub struct UnknownThing {
    ptr: ComPtr<IUnknown>,
}

fn roundtrip<T: ComWrapper + Send + Sync>(p: *mut T::Interface) -> *mut T::Interface {
    unsafe { T::from_raw(p).into_raw() }
}

#[test]
fn id_test() {
    let fake_ptr = 0x10000usize as *mut IUnknown;
    let out = unsafe { UnknownThing::from_raw(fake_ptr).into_raw() };
    assert_eq!(out, fake_ptr);
    assert_eq!(roundtrip::<UnknownThing>(fake_ptr), fake_ptr);
}
