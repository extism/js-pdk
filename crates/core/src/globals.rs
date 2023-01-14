use anyhow::anyhow;
use extism_pdk::bindings::{extism_load_input, extism_log_info};
use extism_pdk::*;
use quickjs_wasm_rs::{Context, Value};
use std::borrow::Cow;
use std::io::{Read, Write};
use std::str;

pub fn inject_globals(
    context: &Context,
) -> anyhow::Result<()>
{
    let global = context.global_object()?;

    // TODO these should proxy to extism_pdk's log functions
    // let console_log_callback = context.wrap_callback(console_log_to(log_stream))?;
    // let console_error_callback = context.wrap_callback(console_log_to(error_stream))?;
    // let console_object = context.object_value()?;
    // console_object.set_property("log", console_log_callback)?;
    // console_object.set_property("error", console_error_callback)?;
    // global.set_property("console", console_object)?;

    let module_obj = context.object_value()?;
    module_obj.set_property("exports", context.null_value()?)?;
    global.set_property("module", module_obj)?;

    // Extism Host object
    let host_object = context.object_value()?;
    let host_input_bytes =
        context.wrap_callback(|ctx: &Context, _this: &Value, args: &[Value]| {
            let input = unsafe { extism_load_input() };
            ctx.array_buffer_value(&input)
        })?;
    host_object.set_property("inputBytes", host_input_bytes)?;
    let host_input_string =
        context.wrap_callback(|ctx: &Context, _this: &Value, args: &[Value]| {
            let input = unsafe { extism_load_input() };
            let string = String::from_utf8(input)?;
            ctx.value_from_str(&string)
        })?;
    host_object.set_property("inputString", host_input_string)?;

    let host_output_bytes =
        context.wrap_callback(|ctx: &Context, _this: &Value, args: &[Value]| {
            let output = args.get(0).unwrap();
            extism_pdk::output(output.as_bytes()?)?;
            ctx.value_from_bool(true)
        })?;
    host_object.set_property("outputBytes", host_output_bytes)?;

    let host_output_string =
        context.wrap_callback(|ctx: &Context, _this: &Value, args: &[Value]| {
            let output = args.get(0).unwrap();
            extism_pdk::output(output.as_str()?)?;
            ctx.value_from_bool(true)
        })?;
    host_object.set_property("outputString", host_output_string)?;

    global.set_property("Host", host_object)?;

    Ok(())
}
