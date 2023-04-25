use std::iter;

use abi_stable::std_types::{ROption, RString};
use anyhow::{bail, Context, Result};

use crate::{
    data::{ForgeResponse, ParamWithType},
    ForgeWorld,
};

use super::call::{
    run_method_no_params, run_method_one_param, run_method_three_params, run_method_two_params,
};

pub(crate) fn get_fn_params(
    given_params: &[&str],
    input_types: &[RString],
) -> Vec<ParamWithType> {
    let params = input_types
        .iter()
        .zip(
            given_params
                .iter()
                .map(|p| Some(*p))
                .chain(iter::repeat(None)),
        )
        .filter_map(|(el_type, el)| ParamWithType::from(el, &el_type).ok())
        .collect::<Vec<_>>();
    params
}

pub(crate) fn get_response<T>(
    w: &mut ForgeWorld,
    api_client_name: &str,
    method_name: &str,
    mut params: Vec<ParamWithType>,
) -> Result<Box<ForgeResponse<T>>> {
    let len = params.len();
    let ret = match len {
        0 => run_method_no_params(w, &api_client_name, &method_name)?,
        1 => match params.pop().context("el1")? {
            ParamWithType::None => {
                run_method_one_param(w, &api_client_name, &method_name, ROption::<RString>::RNone)?
            }
            ParamWithType::Number(el) => run_method_one_param(w, &api_client_name, &method_name, el)?,
            ParamWithType::OptionalNumber(el) => {
                run_method_one_param(w, &api_client_name, &method_name, el)?
            }
            ParamWithType::String(el) => run_method_one_param(w, &api_client_name, &method_name, el)?,
            ParamWithType::OptionalString(el) => {
                run_method_one_param(w, &api_client_name, &method_name, el)?
            }
            _ => bail!("not covered 1 param cases"),
        },
        2 => {
            let el2 = params.pop().context("el2")?;
            let el1 = params.pop().context("el1")?;
            match (el1, el2) {
                (ParamWithType::String(el1), ParamWithType::String(el2)) => {
                    run_method_two_params(w, &api_client_name, &method_name, el1, el2)?
                }
                (ParamWithType::String(el1), ParamWithType::OptionalString(el2)) => {
                    run_method_two_params(w, &api_client_name, &method_name, el1, el2)?
                }
                (ParamWithType::OptionalString(el1), ParamWithType::OptionalNumber(el2)) => {
                    run_method_two_params(w, &api_client_name, &method_name, el1, el2)?
                }
                (ParamWithType::OptionalString(el1), ParamWithType::String(el2)) => {
                    run_method_two_params(w, &api_client_name, &method_name, el1, el2)?
                }
                (ParamWithType::OptionalString(el1), ParamWithType::OptionalString(el2)) => {
                    run_method_two_params(w, &api_client_name, &method_name, el1, el2)?
                }
                (ParamWithType::String(el1), ParamWithType::None) => run_method_two_params(
                    w,
                    &api_client_name,
                    &method_name,
                    el1,
                    ROption::<RString>::RNone,
                )?,
                _ => bail!("not covered all 2 param cases"),
            }
        }
        3 => {
            let el3 = params.pop().context("el3")?;
            let el2 = params.pop().context("el2")?;
            let el1 = params.pop().context("el1")?;
            match (el1, el2, el3) {
                (
                    ParamWithType::OptionalString(el1),
                    ParamWithType::OptionalString(el2),
                    ParamWithType::OptionalDouble(el3),
                ) => run_method_three_params(w, &api_client_name, &method_name, el1, el2, el3)?,
                (ParamWithType::None, ParamWithType::None, ParamWithType::None) => {
                    run_method_three_params(
                        w,
                        &api_client_name,
                        &method_name,
                        ROption::<RString>::RNone,
                        ROption::<RString>::RNone,
                        ROption::<f64>::RNone,
                    )?
                }
                (ParamWithType::OptionalString(el1), ParamWithType::None, ParamWithType::None) => {
                    run_method_three_params(
                        w,
                        &api_client_name,
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
                    &api_client_name,
                    &method_name,
                    el1,
                    el2,
                    ROption::<f64>::RNone,
                )?,
                _ => bail!("not covered all 3 param cases"),
            }
        }
        _ => bail!("Too many arguments"),
    };
    Ok(ret)
}
