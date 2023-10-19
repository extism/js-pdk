use std::str::from_utf8;

use anyhow::anyhow;
use extism_pdk::bindings::extism_load_input;
use extism_pdk::*;
use quickjs_wasm_rs::{Context, Value};

static PRELUDE: &[u8] = include_bytes!("prelude/dist/index.js");

pub fn inject_globals(context: &Context) -> anyhow::Result<()> {
    context.eval_global(
        "prelude.js",
        "globalThis.module = {}; globalThis.module.exports = {}",
    )?;
    // need a *global* var for polyfills to work
    context.eval_global("prelude.js", "global = globalThis")?;
    context.eval_global("prelude.js", from_utf8(PRELUDE)?)?;

    let module = build_module_ojbect(&context)?;
    let console = build_console_object(&context)?;
    let host = build_host_object(&context)?;
    let var = build_var_object(&context)?;
    let http = build_http_object(&context)?;
    let cfg = build_config_object(&context)?;

    let global = context.global_object()?;
    global.set_property("console", console)?;
    global.set_property("module", module)?;
    global.set_property("Host", host)?;
    global.set_property("Var", var)?;
    global.set_property("Http", http)?;
    global.set_property("Config", cfg)?;

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
    let console_warn_callback =
        context.wrap_callback(|ctx: &Context, _this: &Value, args: &[Value]| {
            let stmt = args.get(0).ok_or(anyhow!("Need at least one arg"))?;
            let stmt = stmt.as_str()?;
            warn!("{}", stmt);
            ctx.undefined_value()
        })?;

    let console_object = context.object_value()?;
    console_object.set_property("log", console_log_callback)?;
    console_object.set_property("error", console_error_callback)?;
    console_object.set_property("warn", console_warn_callback)?;

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

fn build_http_object(context: &Context) -> anyhow::Result<Value> {
    let http_req = context.wrap_callback(|ctx: &Context, _this: &Value, args: &[Value]| {
        let req = args
            .get(0)
            .ok_or(anyhow!("Expected http request argument"))?;

        if !req.is_object() {
            anyhow!("First argument should be an http request object");
        }

        let url = req
            .get_property("url")
            .expect("Http Request should have url property");

        let method = req.get_property("method");
        let method_str = match method {
            Ok(m) => m.as_str()?.to_string(),
            Err(..) => "GET".to_string(),
        };

        let mut http_req = HttpRequest::new(url.as_str()?).with_method(method_str);

        let headers = req.get_property("headers");
        if let Ok(headers) = headers {
            if !headers.is_object() {
                anyhow!("Expected headers to be an object");
            }
            let mut header_values = headers.properties()?;
            loop {
                let key = header_values.next_key()?;
                match key {
                    None => break,
                    Some(key) => {
                        let key = key.as_str()?;
                        let value = header_values.next_value()?;
                        let value = value.as_str()?;
                        http_req.headers.insert(key.to_string(), value.to_string());
                    }
                }
            }
        }

        let body_arg = args.get(1).ok_or(ctx.null_value()?).unwrap();
        let mut http_body: Option<String> = None;
        if !body_arg.is_null_or_undefined() {
            let body_arg = body_arg.as_str()?;
            http_body = Some(body_arg.to_string());
        }

        let resp = http::request::<String>(&http_req, http_body)?;

        let body = resp.body();
        let body = from_utf8(&body)?;

        let resp_obj = ctx.object_value()?;
        resp_obj.set_property("body", ctx.value_from_str(body)?);
        resp_obj.set_property("status", ctx.value_from_u32(resp.status_code() as u32)?);

        Ok(resp_obj)
    })?;

    let http_obj = context.object_value()?;
    http_obj.set_property("request", http_req)?;

    Ok(http_obj)
}

fn build_config_object(context: &Context) -> anyhow::Result<Value> {
    let config_get = context.wrap_callback(|ctx: &Context, _this: &Value, args: &[Value]| {
        let key = args.get(0).ok_or(anyhow!("Expected key argument"))?;
        if !key.is_str() {
            anyhow!("Expected key to be a string");
        }

        let key = key.as_str()?;
        match config::get(key) {
            None => ctx.null_value(),
            Some(v) => ctx.value_from_str(v.as_str()),
        }
    })?;

    let config_obj = context.object_value()?;
    config_obj.set_property("get", config_get)?;

    Ok(config_obj)
}
