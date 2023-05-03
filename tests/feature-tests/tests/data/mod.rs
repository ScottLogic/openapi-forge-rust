use std::marker::PhantomData;

use abi_stable::std_types::{ RHashMap, ROption, RString, RVec };
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
#[derive(Debug)]
pub struct FnSignatureInformation {
    pub input_types: RVec<RString>,
    pub return_type: RString,
}

#[derive(Debug, Clone)]
pub enum ParamWithType {
    None,
    Number(i64),
    OptionalNumber(ROption<i64>),
    String(RString),
    OptionalString(ROption<RString>),
    OptionalDouble(ROption<f64>),
    Double(f64),
}

impl ParamWithType {
    pub fn from(el: Option<&str>, el_type: &str) -> anyhow::Result<ParamWithType> {
        if let Some(el) = el {
            let optional_flag = el_type.contains("Option<");
            if el_type.contains("i64") {
                let el = el.parse::<i64>().context("parse fail")?;
                match optional_flag {
                    true => Ok(ParamWithType::OptionalNumber(ROption::RSome(el))),
                    false => Ok(ParamWithType::Number(el)),
                }
            } else if el_type.contains("f64") {
                let el = el.parse::<f64>().context("parse fail")?;
                match optional_flag {
                    true => Ok(ParamWithType::OptionalDouble(ROption::RSome(el))),
                    false => Ok(ParamWithType::Double(el)),
                }
            } else {
                match optional_flag {
                    true => Ok(ParamWithType::OptionalString(ROption::RSome(el.into()))),
                    false => Ok(ParamWithType::String(el.into())),
                }
            }
        } else {
            Ok(ParamWithType::None)
        }
    }
}