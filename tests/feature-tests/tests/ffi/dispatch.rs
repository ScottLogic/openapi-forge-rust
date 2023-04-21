use std::iter;

use abi_stable::std_types::{RVec, RString, ROption};
use anyhow::{Result, bail};

use crate::{data::{ParamWithType, ForgeResponse}, ForgeWorld};

use super::call::{run_method_no_params, run_method_one_param, run_method_two_params, run_method_three_params};


pub(crate) fn get_fn_params(given_params: Vec<&str>, input_types: RVec<RString>) -> Vec<ParamWithType> {
    let params = input_types
        .into_iter()
        .zip(
            given_params
                .into_iter()
                .map(|p| Some(p))
                .chain(iter::repeat(None)),
        )
        .filter_map(|(el_type, el)| ParamWithType::from(el, &el_type[..]).ok())
        .collect::<Vec<_>>();
    params
}

pub(crate) fn get_response<T>(
    w: &mut ForgeWorld,
    client_name: &str,
    method_name: &str,
    params: Vec<ParamWithType>,
) -> Result<Box<ForgeResponse<T>>> {
    let ret = match params.len() {
        0 => run_method_no_params(w, &client_name, &method_name)?,
        1 => match params[0].clone() {
            ParamWithType::None => {
                run_method_one_param(w, &client_name, &method_name, ROption::<RString>::RNone)?
            }
            ParamWithType::Number(el) => run_method_one_param(w, &client_name, &method_name, el)?,
            ParamWithType::OptionalNumber(el) => {
                run_method_one_param(w, &client_name, &method_name, el)?
            }
            ParamWithType::String(el) => run_method_one_param(w, &client_name, &method_name, el)?,
            ParamWithType::OptionalString(el) => {
                run_method_one_param(w, &client_name, &method_name, el)?
            }
            _ => bail!("not covered 1 param cases"),
        },
        2 => match (params[0].clone(), params[1].clone()) {
            (ParamWithType::String(el1), ParamWithType::String(el2)) => {
                run_method_two_params(w, &client_name, &method_name, el1, el2)?
            }
            (ParamWithType::String(el1), ParamWithType::OptionalString(el2)) => {
                run_method_two_params(w, &client_name, &method_name, el1, el2)?
            }
            (ParamWithType::OptionalString(el1), ParamWithType::OptionalNumber(el2)) => {
                run_method_two_params(w, &client_name, &method_name, el1, el2)?
            }
            (ParamWithType::OptionalString(el1), ParamWithType::String(el2)) => {
                run_method_two_params(w, &client_name, &method_name, el1, el2)?
            }
            (ParamWithType::OptionalString(el1), ParamWithType::OptionalString(el2)) => {
                run_method_two_params(w, &client_name, &method_name, el1, el2)?
            }
            (ParamWithType::String(el1), ParamWithType::None) => run_method_two_params(
                w,
                &client_name,
                &method_name,
                el1,
                ROption::<RString>::RNone,
            )?,
            _ => bail!("not covered all 2 param cases"),
        },
        3 => match (params[0].clone(), params[1].clone(), params[2].clone()) {
            (
                ParamWithType::OptionalString(el1),
                ParamWithType::OptionalString(el2),
                ParamWithType::OptionalDouble(el3),
            ) => run_method_three_params(w, &client_name, &method_name, el1, el2, el3)?,
            (ParamWithType::None, ParamWithType::None, ParamWithType::None) => {
                run_method_three_params(
                    w,
                    &client_name,
                    &method_name,
                    ROption::<RString>::RNone,
                    ROption::<RString>::RNone,
                    ROption::<f64>::RNone,
                )?
            }
            (ParamWithType::OptionalString(el1), ParamWithType::None, ParamWithType::None) => {
                run_method_three_params(
                    w,
                    &client_name,
                    &method_name,
                    el1,
                    ROption::<RString>::RNone,
                    ROption::<f64>::RNone,
                )?
            }
            (
                ParamWithType::OptionalString(el1),
                ParamWithType::OptionalString(el2),
                ParamWithType::None,
            ) => run_method_three_params(
                w,
                &client_name,
                &method_name,
                el1,
                el2,
                ROption::<f64>::RNone,
            )?,
            _ => bail!("not covered all 3 param cases"),
        },
        _ => bail!("Too many arguments"),
    };
    Ok(ret)
}