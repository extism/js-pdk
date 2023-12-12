use std::{borrow::Cow, collections::HashMap, str::from_utf8};

use anyhow::{anyhow, bail, Context};
use extism_pdk::extism::load_input;
use extism_pdk::*;
use quickjs_wasm_rs::{JSContextRef, JSError, JSValue, JSValueRef};

static PRELUDE: &[u8] = include_bytes!("prelude/dist/index.js");

pub fn inject_globals(context: &JSContextRef) -> anyhow::Result<()> {
    let module = build_module_ojbect(&context)?;
    let console = build_console_object(&context)?;
    let host = build_host_object(&context)?;
    let var = build_var_object(&context)?;
    let http = build_http_object(&context)?;
    let cfg = build_config_object(&context)?;
    let decoder = build_decoder(&context)?;
    let encoder = build_encoder(&context)?;

    let global = context.global_object()?;
    global.set_property("console", console)?;
    global.set_property("module", module)?;
    global.set_property("Host", host)?;
    global.set_property("Var", var)?;
    global.set_property("Http", http)?;
    global.set_property("Config", cfg)?;
    global.set_property("__decodeUtf8BufferToString", decoder)?;
    global.set_property("__encodeStringToUtf8Buffer", encoder)?;

    context.eval_global(
        "script.js",
        "globalThis.module = {}; globalThis.module.exports = {}",
    )?;
    // need a *global* var for polyfills to work
    context.eval_global("script.js", "global = globalThis")?;
    context.eval_global("script.js", from_utf8(PRELUDE)?)?;

    Ok(())
}
fn build_console_object(context: &JSContextRef) -> anyhow::Result<JSValueRef> {
    let console_log_callback = context.wrap_callback(
        |_ctx: &JSContextRef, _this: JSValueRef, args: &[JSValueRef]| {
            let stmt = args.get(0).ok_or(anyhow!("Need at least one arg"))?;
            let stmt = stmt.as_str()?;
            info!("{}", stmt);
            Ok(JSValue::Undefined)
        },
    )?;
    let console_error_callback = context.wrap_callback(
        |_ctx: &JSContextRef, _this: JSValueRef, args: &[JSValueRef]| {
            let stmt = args.get(0).ok_or(anyhow!("Need at least one arg"))?;
            let stmt = stmt.as_str()?;
            error!("{}", stmt);
            Ok(JSValue::Undefined)
        },
    )?;
    let console_warn_callback = context.wrap_callback(
        |_ctx: &JSContextRef, _this: JSValueRef, args: &[JSValueRef]| {
            let stmt = args.get(0).ok_or(anyhow!("Need at least one arg"))?;
            let stmt = stmt.as_str()?;
            warn!("{}", stmt);
            Ok(JSValue::Undefined)
        },
    )?;

    let console_object = context.object_value()?;
    console_object.set_property("log", console_log_callback)?;
    console_object.set_property("error", console_error_callback)?;
    console_object.set_property("warn", console_warn_callback)?;

    Ok(console_object)
}

fn build_module_ojbect(context: &JSContextRef) -> anyhow::Result<JSValueRef> {
    let exports = context.object_value()?;
    let module_obj = context.object_value()?;
    module_obj.set_property("exports", exports)?;
    Ok(module_obj)
}

fn build_host_object(context: &JSContextRef) -> anyhow::Result<JSValueRef> {
    let host_input_bytes = context.wrap_callback(
        |_ctx: &JSContextRef, _this: JSValueRef, _args: &[JSValueRef]| {
            let input = unsafe { load_input() };
            Ok(JSValue::ArrayBuffer(input))
        },
    )?;
    let host_input_string = context.wrap_callback(
        |_ctx: &JSContextRef, _this: JSValueRef, _args: &[JSValueRef]| {
            let input = unsafe { load_input() };
            let string = String::from_utf8(input)?;
            Ok(JSValue::String(string))
        },
    )?;
    let host_output_bytes = context.wrap_callback(
        |_ctx: &JSContextRef, _this: JSValueRef, args: &[JSValueRef]| {
            let output = args.get(0).unwrap();
            extism_pdk::output(output.as_bytes()?)?;
            Ok(JSValue::Bool(true))
        },
    )?;
    let host_output_string = context.wrap_callback(
        |_ctx: &JSContextRef, _this: JSValueRef, args: &[JSValueRef]| {
            let output = args.get(0).unwrap();
            extism_pdk::output(output.as_str()?)?;
            Ok(JSValue::Bool(true))
        },
    )?;

    let host_object = context.object_value()?;
    host_object.set_property("inputBytes", host_input_bytes)?;
    host_object.set_property("inputString", host_input_string)?;
    host_object.set_property("outputBytes", host_output_bytes)?;
    host_object.set_property("outputString", host_output_string)?;

    Ok(host_object)
}

fn build_var_object(context: &JSContextRef) -> anyhow::Result<JSValueRef> {
    let var_set = context.wrap_callback(
        |_ctx: &JSContextRef, _this: JSValueRef, args: &[JSValueRef]| {
            let var_name = args.get(0).ok_or(anyhow!("Expected var_name argument"))?;
            let data = args.get(1).ok_or(anyhow!("Expected data argument"))?;

            if data.is_str() {
                var::set(var_name.as_str()?, data.as_str()?)?;
            } else if data.is_array_buffer() {
                var::set(var_name.as_str()?, data.as_bytes()?)?;
            }

            Ok(JSValue::Undefined)
        },
    )?;
    let var_get = context.wrap_callback(
        |_ctx: &JSContextRef, _this: JSValueRef, args: &[JSValueRef]| {
            let var_name = args.get(0).ok_or(anyhow!("Expected var_name argument"))?;
            let data = var::get::<Vec<u8>>(var_name.as_str()?)?;
            match data {
                Some(d) => Ok(JSValue::ArrayBuffer(d)),
                None => Ok(JSValue::Null),
            }
        },
    )?;

    let var_get_str = context.wrap_callback(
        |_ctx: &JSContextRef, _this: JSValueRef, args: &[JSValueRef]| {
            let var_name = args.get(0).ok_or(anyhow!("Expected var_name argument"))?;
            let data = var::get::<String>(var_name.as_str()?)?;
            match data {
                Some(d) => Ok(JSValue::String(d)),
                None => Ok(JSValue::Null),
            }
        },
    )?;

    let var_object = context.object_value()?;
    var_object.set_property("set", var_set)?;
    var_object.set_property("getBytes", var_get)?;
    var_object.set_property("getString", var_get_str)?;

    Ok(var_object)
}

fn build_http_object(context: &JSContextRef) -> anyhow::Result<JSValueRef> {
    let http_req = context.wrap_callback(
        |ctx: &JSContextRef, _this: JSValueRef, args: &[JSValueRef]| {
            let req = args
                .get(0)
                .ok_or(anyhow!("Expected http request argument"))?;

            if !req.is_object() {
                bail!("First argument should be an http request object");
            }

            let url = req
                .get_property("url")
                .context("Http Request should have url property")?;

            let method = req.get_property("method");
            let method_str = match method {
                Ok(m) => m.as_str()?.to_string(),
                Err(..) => "GET".to_string(),
            };

            let mut http_req = HttpRequest::new(url.as_str()?).with_method(method_str);

            let headers = req.get_property("headers")?;
            if !headers.is_null_or_undefined() {
                if !headers.is_object() {
                    bail!("Expected headers to be an object");
                }
                if !headers.is_object() {
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
            }

            let body_arg = args.get(1);
            let mut http_body: Option<String> = None;
            if let Some(body) = body_arg {
                http_body = Some(body.as_str()?.to_string());
            }

            let resp = http::request::<String>(&http_req, http_body)?;
            let body = resp.body();
            let body = from_utf8(&body)?;

            let mut resp_obj = HashMap::new();
            resp_obj.insert("body".to_string(), JSValue::String(body.into()));
            resp_obj.insert(
                "status".to_string(),
                JSValue::Int(resp.status_code() as i32),
            );

            Ok(JSValue::Object(resp_obj))
        },
    )?;

    let http_obj = context.object_value()?;
    http_obj.set_property("request", http_req)?;

    Ok(http_obj)
}

fn build_config_object(context: &JSContextRef) -> anyhow::Result<JSValueRef> {
    let config_get = context.wrap_callback(
        |_ctx: &JSContextRef, _this: JSValueRef, args: &[JSValueRef]| {
            let key = args.get(0).ok_or(anyhow!("Expected key argument"))?;
            if !key.is_str() {
                bail!("Expected key to be a string");
            }

            let key = key.as_str()?;
            match config::get(key) {
                Ok(Some(v)) => Ok(JSValue::String(v)),
                _ => Ok(JSValue::Null),
            }
        },
    )?;

    let config_obj = context.object_value()?;
    config_obj.set_property("get", config_get)?;

    Ok(config_obj)
}

fn build_decoder(context: &JSContextRef) -> anyhow::Result<JSValueRef> {
    Ok(context.wrap_callback(decode_utf8_buffer_to_js_string())?)
}

fn build_encoder(context: &JSContextRef) -> anyhow::Result<JSValueRef> {
    Ok(context.wrap_callback(encode_js_string_to_utf8_buffer())?)
}

fn decode_utf8_buffer_to_js_string(
) -> impl FnMut(&JSContextRef, JSValueRef, &[JSValueRef]) -> anyhow::Result<JSValue> {
    move |_ctx: &JSContextRef, _this: JSValueRef, args: &[JSValueRef]| {
        if args.len() != 5 {
            return Err(anyhow!("Expecting 5 arguments, received {}", args.len()));
        }

        let buffer: Vec<u8> = args[0].try_into()?;
        let byte_offset: usize = args[1].try_into()?;
        let byte_length: usize = args[2].try_into()?;
        let fatal: bool = args[3].try_into()?;
        let ignore_bom: bool = args[4].try_into()?;

        let mut view = buffer
            .get(byte_offset..(byte_offset + byte_length))
            .ok_or_else(|| {
                anyhow!("Provided offset and length is not valid for provided buffer")
            })?;

        if !ignore_bom {
            view = match view {
                // [0xEF, 0xBB, 0xBF] is the UTF-8 BOM which we want to strip
                [0xEF, 0xBB, 0xBF, rest @ ..] => rest,
                _ => view,
            };
        }

        let str =
            if fatal {
                Cow::from(from_utf8(view).map_err(|_| {
                    JSError::Type("The encoded data was not valid utf-8".to_string())
                })?)
            } else {
                String::from_utf8_lossy(view)
            };
        Ok(str.to_string().into())
    }
}

fn encode_js_string_to_utf8_buffer(
) -> impl FnMut(&JSContextRef, JSValueRef, &[JSValueRef]) -> anyhow::Result<JSValue> {
    move |_ctx: &JSContextRef, _this: JSValueRef, args: &[JSValueRef]| {
        if args.len() != 1 {
            return Err(anyhow!("Expecting 1 argument, got {}", args.len()));
        }

        let js_string: String = args[0].try_into()?;
        Ok(js_string.into_bytes().into())
    }
}
