use std::marker::PhantomData;

use abi_stable::std_types::RString;

#[derive(Debug)]
#[repr(C)]
pub struct ApiClient {
    _private: [u8; 0],
}

#[derive(Debug)]
#[repr(C)]
pub struct Configuration {
    _private: [u8; 0],
}

#[derive(Debug)]
#[repr(C)]
pub struct Client {
    _private: [u8; 0],
}

#[derive(Debug)]
#[repr(C)]
pub struct ForgeResponse<T> {
    _private: [u8; 0],
    _phantom: PhantomData<T>,
}

#[derive(Debug)]
#[repr(C)]
pub struct FFIObject {
    _private: [u8; 0],
}

#[repr(C)]
#[derive(Debug)]
pub struct FFISafeResponseTuple<T> {
    pub o: Box<ForgeResponse<T>>,
    pub serialized: RString,
}