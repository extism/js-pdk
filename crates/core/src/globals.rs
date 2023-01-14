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

    let module = create_module_ojbect(&context)?;
    let host = create_host_object(&context)?;

    global.set_property("module", module)?;
    global.set_property("Host", host)?;

    Ok(())
}

fn create_module_ojbect(context: &Context) -> anyhow::Result<Value> {
    let module_obj = context.object_value()?;
    let exports = context.object_value()?;
    module_obj.set_property("exports", exports)?;
    Ok(module_obj)
}

fn create_host_object(context: &Context) -> anyhow::Result<Value> {
    let host_object = context.object_value()?;
    let host_input_bytes =
        context.wrap_callback(|ctx: &Context, _this: &Value, args: &[Value]| {
            let input = unsafe { extism_load_input() };
            ctx.array_buffer_value(&input)
        })?;
    let host_input_string =
        context.wrap_callback(|ctx: &Context, _this: &Value, args: &[Value]| {
            let input = unsafe { extism_load_input() };
            let string = String::from_utf8(input)?;
            ctx.value_from_str(&string)
        })?;
    let host_output_bytes =
        context.wrap_callback(|ctx: &Context, _this: &Value, args: &[Value]| {
            let output = args.get(0).unwrap();
            extism_pdk::output(output.as_bytes()?)?;
            ctx.value_from_bool(true)
        })?;
    let host_output_string =
        context.wrap_callback(|ctx: &Context, _this: &Value, args: &[Value]| {
            let output = args.get(0).unwrap();
            extism_pdk::output(output.as_str()?)?;
            ctx.value_from_bool(true)
        })?;

    host_object.set_property("inputBytes", host_input_bytes)?;
    host_object.set_property("inputString", host_input_string)?;
    host_object.set_property("outputBytes", host_output_bytes)?;
    host_object.set_property("outputString", host_output_string)?;

    Ok(host_object)
}