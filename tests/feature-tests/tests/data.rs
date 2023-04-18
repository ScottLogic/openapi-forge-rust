use std::marker::PhantomData;

use abi_stable::std_types::{RHashMap, RString};

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
pub struct FFISafeTuple<T>(pub Box<ForgeResponse<T>>, pub RString);

#[repr(C)]
#[derive(Debug)]
pub struct ObjectTypeInformation {
    pub name: RString,
    pub fields: RHashMap<RString, RString>,
}
