use anyhow::anyhow;
use extism_pdk::bindings::extism_load_input;
use extism_pdk::*;
use quickjs_wasm_rs::{Context, Value};

pub fn inject_globals(context: &Context) -> anyhow::Result<()> {
    let module = build_module_ojbect(&context)?;
    let console = build_console_object(&context)?;
    let host = build_host_object(&context)?;
    let var = build_var_object(&context)?;

    let global = context.global_object()?;
    global.set_property("console", console)?;
    global.set_property("module", module)?;
    global.set_property("Host", host)?;
    global.set_property("Var", var)?;

    Ok(())
}

fn build_console_object(context: &Context) -> anyhow::Result<Value> {
    let console_log_callback =
        context.wrap_callback(|ctx: &Context, _this: &Value, args: &[Value]| {
            let stmt = args.get(0).ok_or(anyhow!("Need at least one arg"))?;
            let stmt = stmt.as_str()?;
            info!("{}", stmt);
            ctx.undefined_value()
        })?;
    let console_error_callback =
        context.wrap_callback(|ctx: &Context, _this: &Value, args: &[Value]| {
            let stmt = args.get(0).ok_or(anyhow!("Need at least one arg"))?;
            let stmt = stmt.as_str()?;
            error!("{}", stmt);
            ctx.undefined_value()
        })?;

    let console_object = context.object_value()?;
    console_object.set_property("log", console_log_callback)?;
    console_object.set_property("error", console_error_callback)?;

    Ok(console_object)
}

fn build_module_ojbect(context: &Context) -> anyhow::Result<Value> {
    let exports = context.object_value()?;
    let module_obj = context.object_value()?;
    module_obj.set_property("exports", exports)?;
    Ok(module_obj)
}

fn build_host_object(context: &Context) -> anyhow::Result<Value> {
    let host_input_bytes =
        context.wrap_callback(|ctx: &Context, _this: &Value, _args: &[Value]| {
            let input = unsafe { extism_load_input() };
            ctx.array_buffer_value(&input)
        })?;
    let host_input_string =
        context.wrap_callback(|ctx: &Context, _this: &Value, _args: &[Value]| {
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

    let host_object = context.object_value()?;
    host_object.set_property("inputBytes", host_input_bytes)?;
    host_object.set_property("inputString", host_input_string)?;
    host_object.set_property("outputBytes", host_output_bytes)?;
    host_object.set_property("outputString", host_output_string)?;

    Ok(host_object)
}

fn build_var_object(context: &Context) -> anyhow::Result<Value> {
    let var_set = context.wrap_callback(|ctx: &Context, _this: &Value, args: &[Value]| {
        let var_name = args.get(0).ok_or(anyhow!("Expected var_name argument"))?;
        let data = args.get(1).ok_or(anyhow!("Expected data argument"))?;

        if data.is_str() {
            var::set(var_name.as_str()?, data.as_str()?)?;
        } else if data.is_array_buffer() {
            var::set(var_name.as_str()?, data.as_bytes()?)?;
        }

        ctx.undefined_value()
    })?;
    let var_get = context.wrap_callback(|ctx: &Context, _this: &Value, args: &[Value]| {
        let var_name = args.get(0).ok_or(anyhow!("Expected var_name argument"))?;
        let data = var::get::<Vec<u8>>(var_name.as_str()?)?;
        match data {
            Some(d) => ctx.array_buffer_value(d.as_slice()),
            None => ctx.null_value(),
        }
    })?;

    let var_get_str = context.wrap_callback(|ctx: &Context, _this: &Value, args: &[Value]| {
        let var_name = args.get(0).ok_or(anyhow!("Expected var_name argument"))?;
        let data = var::get::<String>(var_name.as_str()?)?;
        match data {
            Some(d) => ctx.value_from_str(d.as_str()),
            None => ctx.null_value(),
        }
    })?;

    let var_object = context.object_value()?;
    var_object.set_property("set", var_set)?;
    var_object.set_property("getBytes", var_get)?;
    var_object.set_property("getString", var_get_str)?;

    Ok(var_object)
}
