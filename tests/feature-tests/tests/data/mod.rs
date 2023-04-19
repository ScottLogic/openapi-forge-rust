use std::marker::PhantomData;

use abi_stable::std_types::{ RHashMap, RString, RVec, ROption };
use anyhow::Context;

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

#[repr(C)]
pub struct FnSignatureInformation {
    pub input_types: RVec<RString>,
    pub return_type: RString,
}

#[derive(Debug, Clone)]
pub enum ParamWithType {
    Number(i32),
    OptionalNumber(ROption<i32>),
    String(RString),
    OptionalString(ROption<RString>),
}

impl ParamWithType {
    pub fn from(el: &str, el_type: &str) -> anyhow::Result<ParamWithType> {
        let optional_flag = el_type.contains("Option<");
        if el_type.contains("i32") {
            let el = el.parse::<i32>().context("parse fail")?;
            match optional_flag {
                true => Ok(ParamWithType::OptionalNumber(ROption::RSome(el))),
                false => Ok(ParamWithType::Number(el)),
            }
        } else {
            match optional_flag {
                true => Ok(ParamWithType::OptionalString(ROption::RSome(el.into()))),
                false => Ok(ParamWithType::String(el.into())),
            }
        }
    }
}