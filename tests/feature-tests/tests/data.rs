use std::marker::PhantomData;

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