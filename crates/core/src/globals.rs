use std::{borrow::Cow, collections::HashMap, str::from_utf8};

use anyhow::{anyhow, bail, Context};
use chrono::{SecondsFormat, Utc};
use extism_pdk::extism::load_input;
use extism_pdk::*;
use quickjs_wasm_rs::{JSContextRef, JSError, JSValue, JSValueRef};
use rquickjs::{
    function::{Func, MutFn},
    markers::Invariant,
    Context as JSContext, Function,
};


static PRELUDE: &[u8] = include_bytes!("prelude/dist/index.js"); // if this panics, run `make` from the root

pub fn inject_globals(context: &JSContext) -> anyhow::Result<()> {
    
    context.with(|this| {
        let module = build_module_object(this.clone())?;
        
        let console = build_console_object(this.clone())?;
        let var = build_var_object(this.clone())?;
        let http = build_http_object(this.clone())?;
        let cfg = build_config_object(this.clone())?;
        let decoder = build_decoder(context)?;
        let encoder = build_encoder(context)?;
        let clock = build_clock(context)?;
        let mem = build_memory(context)?;
        let host = build_host_object(this.clone())?;
    
        let global = this.globals()
        global.set("console", console)?;
        global.set("module", module)?;
        global.set("Host", host)?;
        global.set("Var", var)?;
        global.set("Http", http)?;
        global.set("Config", cfg)?;
        global.set("Memory", mem)?;
        global.set("__decodeUtf8BufferToString", decoder)?;
        global.set("__encodeStringToUtf8Buffer", encoder)?;
        global.set("__getTime", clock)?;
    
        add_host_functions(this.clone())?;
    
        this.eval_global(
            "script.js",
            "globalThis.module = {}; globalThis.module.exports = {}",
        )?;
        // need a *global* var for polyfills to work
        context.eval_global("script.js", "global = globalThis")?;
        context.eval_global("script.js", from_utf8(PRELUDE)?)?;

        Ok(())
    })?;
    

    Ok(())
}

#[link(wasm_import_module = "shim")]
extern "C" {
    // this import will get satisified by the import shim
    fn __invokeHostFunc_0_0(func_idx: u32);
    fn __invokeHostFunc_1_0(func_idx: u32, ptr: u64);
    fn __invokeHostFunc_2_0(func_idx: u32, ptr: u64, ptr2: u64);
    fn __invokeHostFunc_3_0(func_idx: u32, ptr: u64, ptr2: u64, ptr3: u64);
    fn __invokeHostFunc_4_0(func_idx: u32, ptr: u64, ptr2: u64, ptr3: u64, ptr4: u64);
    fn __invokeHostFunc_5_0(func_idx: u32, ptr: u64, ptr2: u64, ptr3: u64, ptr4: u64, ptr5: u64);
    fn __invokeHostFunc_0_1(func_idx: u32) -> u64;
    fn __invokeHostFunc_1_1(func_idx: u32, ptr: u64) -> u64;
    fn __invokeHostFunc_2_1(func_idx: u32, ptr: u64, ptr2: u64) -> u64;
    fn __invokeHostFunc_3_1(func_idx: u32, ptr: u64, ptr2: u64, ptr3: u64) -> u64;
    fn __invokeHostFunc_4_1(func_idx: u32, ptr: u64, ptr2: u64, ptr3: u64, ptr4: u64) -> u64;
    fn __invokeHostFunc_5_1(
        func_idx: u32,
        ptr: u64,
        ptr2: u64,
        ptr3: u64,
        ptr4: u64,
        ptr5: u64,
    ) -> u64;
}

fn get_args_as_str(args: &rquickjs::prelude::Rest<rquickjs::Value>) -> anyhow::Result<String> {
    args.iter()
        .map(|arg| {
            arg.as_string()
                .ok_or(rquickjs::Error::Unknown)
                .and_then(|s| s.to_string())
        })
        .collect::<Result<Vec<String>, _>>()
        .map(|vec| vec.join(" "))
        .context("Failed to convert args to string")
}

fn to_js_error(cx: rquickjs::Ctx, e: anyhow::Error) -> rquickjs::Error {
    match e.downcast::<rquickjs::Error>() {
        Ok(e) => e,
        Err(e) => cx.throw(rquickjs::Value::from_exception(
            rquickjs::Exception::from_message(cx.clone(), &e.to_string())
                .expect("Creating an exception should succeed"),
        )),
    }
}

fn build_console_object(this: rquickjs::Ctx) -> anyhow::Result<rquickjs::Object> {
        let console = rquickjs::Object::new(this.clone())?;
        let console_info_callback = Function::new(
            this.clone(),
            MutFn::new(move |cx, args| {
                let statement = get_args_as_str(&args).map_err(|e| to_js_error(cx, e))?;
                info!("{}", statement);
                Ok::<_, rquickjs::Error>(())
            }),
        )?;
        console.set("log", console_info_callback.clone())?;
        console.set("info", console_info_callback)?;

        console.set(
            "error",
            Function::new(
                this.clone(),
                MutFn::new(move |cx, args| {
                    let statement = get_args_as_str(&args).map_err(|e| to_js_error(cx, e))?;
                    warn!("{}", statement);
                    Ok::<_, rquickjs::Error>(())
                }),
            ),
        )?;

        console.set(
            "debug",
            Function::new(
                this.clone(),
                MutFn::new(move |cx, args| {
                    let statement = get_args_as_str(&args).map_err(|e| to_js_error(cx, e))?;
                    debug!("{}", &statement);
                    Ok::<_, rquickjs::Error>(())
                }),
            ),
        )?;

    Ok(console)
}

fn build_module_object(this: rquickjs::Ctx) -> anyhow::Result<rquickjs::Object> {

let exports = rquickjs::Object::new(this.clone())?;
    let module = rquickjs::Object::new(this.clone())?;
    module.set("exports", exports)?;
    
    Ok(module)
}

fn build_host_object(this: rquickjs::Ctx) -> anyhow::Result<rquickjs::Object> {
        let host_input_bytes = rquickjs::Function::new(
            this.clone(),
            MutFn::new(move |cx| {
                let input = unsafe { load_input() };
                Ok::<_, rquickjs::Error>(rquickjs::ArrayBuffer::new(cx, input))
            }),
        )?;
        let host_input_string = rquickjs::Function::new(
            this.clone(),
            MutFn::new(move |cx| {
                let input = unsafe { load_input() };
                let input_string = String::from_utf8(input)?;
                rquickjs::String::from_str(cx, &input_string)
            }),
        )?;
        let host_output_bytes = rquickjs::Function::new(
            this.clone(),
            MutFn::new(move |cx, args: rquickjs::prelude::Rest<rquickjs::Value>| {
                let output = args.first().unwrap().clone();
                let output_buffer = rquickjs::ArrayBuffer::from_value(output).unwrap();
                extism_pdk::output(output_buffer.as_bytes());
                Ok::<_, rquickjs::Error>(rquickjs::Value::new_bool(cx, true))
            }),
        )?;
        let host_output_string = rquickjs::Function::new(
            this.clone(),
            MutFn::new(move |cx, args: rquickjs::prelude::Rest<rquickjs::Value>| {
                let output = args.first().unwrap().clone();
                let output_string = output
                    .as_string()
                    .ok_or(rquickjs::Error::Unknown)?
                    .to_string()?;
                extism_pdk::output(output_string);
                Ok::<_, rquickjs::Error>(rquickjs::Value::new_bool(cx, true))
            }),
        )?;
        let host_object = rquickjs::Object::new(this.clone())?;
        host_object.set("inputBytes", host_input_bytes)?;
        host_object.set("inputString", host_input_string)?;
        host_object.set("outputBytes", host_output_bytes)?;
        host_object.set("outputString", host_output_string)?;
    Ok(host_object)
}


fn add_host_functions(this: rquickjs::Ctx<'_>) -> anyhow::Result<()> {
        let globals = this.globals();
        let host_object = globals.get::<_, rquickjs::Object>("host")?;
        let invoke_host = host_object.get::<_, rquickjs::Value>("invokeHost")?;
        if invoke_host.is_null() || invoke_host.is_undefined() {
            let host_invoke_func = rquickjs::Function::new(
                this.clone(),
                move |cx, args: rquickjs::prelude::Rest<rquickjs::Value<'_>>| {
                    let func_id = args.first().unwrap().as_int().unwrap() as u32;
                    let len = args.len() - 1;
                    match len {
                        0 => {
                            let result = unsafe { __invokeHostFunc_0_1(func_id) };

                            Ok(rquickjs::Value::new_float(cx, result as f64))
                        }
                        1 => {
                            let ptr = args.get(1).unwrap().as_float().unwrap();
                            let result = unsafe { __invokeHostFunc_1_1(func_id, ptr as u64) };
                            Ok(rquickjs::Value::new_float(cx, result as f64))
                        }
                        2 => {
                            let ptr = args.get(1).unwrap().as_float().unwrap();
                            let ptr2 = args.get(2).unwrap().as_float().unwrap();
                            let result =
                                unsafe { __invokeHostFunc_2_1(func_id, ptr as u64, ptr2 as u64) };
                            Ok(rquickjs::Value::new_float(cx, result as f64))
                        }
                        3 => {
                            let ptr = args.get(1).unwrap().as_float().unwrap();
                            let ptr2 = args.get(2).unwrap().as_float().unwrap();
                            let ptr3 = args.get(3).unwrap().as_float().unwrap();
                            let result = unsafe {
                                __invokeHostFunc_3_1(func_id, ptr as u64, ptr2 as u64, ptr3 as u64)
                            };
                            Ok(rquickjs::Value::new_float(cx, result as f64))
                        }
                        4 => {
                            let ptr = args.get(1).unwrap().as_float().unwrap();
                            let ptr2 = args.get(2).unwrap().as_float().unwrap();
                            let ptr3 = args.get(3).unwrap().as_float().unwrap();
                            let ptr4 = args.get(4).unwrap().as_float().unwrap();
                            let result = unsafe {
                                __invokeHostFunc_4_1(
                                    func_id,
                                    ptr as u64,
                                    ptr2 as u64,
                                    ptr3 as u64,
                                    ptr4 as u64,
                                )
                            };
                            Ok(rquickjs::Value::new_float(cx, result as f64))
                        }
                        5 => {
                            let ptr = args.get(1).unwrap().as_float().unwrap();
                            let ptr2 = args.get(2).unwrap().as_float().unwrap();
                            let ptr3 = args.get(3).unwrap().as_float().unwrap();
                            let ptr4 = args.get(4).unwrap().as_float().unwrap();
                            let ptr5 = args.get(5).unwrap().as_float().unwrap();
                            let result = unsafe {
                                __invokeHostFunc_5_1(
                                    func_id,
                                    ptr as u64,
                                    ptr2 as u64,
                                    ptr3 as u64,
                                    ptr4 as u64,
                                    ptr5 as u64,
                                )
                            };
                            Ok(rquickjs::Value::new_float(cx, result as f64))
                        }
                        n => Err(to_js_error(
                            cx,
                            anyhow!("__invokeHostFunc with {n} parameters is not implemented"),
                        )),
                    }
                },
            )?;
            let host_invoke_func0 = rquickjs::Function::new(
                this.clone(),
                move |cx: rquickjs::Ctx, args: rquickjs::prelude::Rest<rquickjs::Value>| {
                    let func_id = args.first().unwrap().as_int().unwrap() as u32;
                    let len = args.len() - 1;
                    match len {
                        0 => {
                            unsafe { __invokeHostFunc_0_0(func_id) };
                        }
                        1 => {
                            let ptr = args.get(1).unwrap().as_float().unwrap();
                            unsafe { __invokeHostFunc_1_0(func_id, ptr as u64) };
                        }
                        2 => {
                            let ptr = args.get(1).unwrap().as_float().unwrap();
                            let ptr2 = args.get(2).unwrap().as_float().unwrap();
                            unsafe { __invokeHostFunc_2_0(func_id, ptr as u64, ptr2 as u64) };
                        }
                        3 => {
                            let ptr = args.get(1).unwrap().as_float().unwrap();
                            let ptr2 = args.get(2).unwrap().as_float().unwrap();
                            let ptr3 = args.get(3).unwrap().as_float().unwrap();
                            unsafe {
                                __invokeHostFunc_3_0(func_id, ptr as u64, ptr2 as u64, ptr3 as u64)
                            };
                        }
                        4 => {
                            let ptr = args.get(1).unwrap().as_float().unwrap();
                            let ptr2 = args.get(2).unwrap().as_float().unwrap();
                            let ptr3 = args.get(3).unwrap().as_float().unwrap();
                            let ptr4 = args.get(4).unwrap().as_float().unwrap();
                            unsafe {
                                __invokeHostFunc_4_0(
                                    func_id,
                                    ptr as u64,
                                    ptr2 as u64,
                                    ptr3 as u64,
                                    ptr4 as u64,
                                )
                            };
                        }
                        5 => {
                            let ptr = args.get(1).unwrap().as_float().unwrap();
                            let ptr2 = args.get(2).unwrap().as_float().unwrap();
                            let ptr3 = args.get(3).unwrap().as_float().unwrap();
                            let ptr4 = args.get(4).unwrap().as_float().unwrap();
                            let ptr5 = args.get(5).unwrap().as_float().unwrap();
                            unsafe {
                                __invokeHostFunc_5_0(
                                    func_id,
                                    ptr as u64,
                                    ptr2 as u64,
                                    ptr3 as u64,
                                    ptr4 as u64,
                                    ptr5 as u64,
                                )
                            };
                        }
                        n => {
                            return Err(to_js_error(
                                cx,
                                anyhow!("__invokeHostFunc with {n} parameters is not implemented"),
                            ))
                        }
                    }
                    Ok(rquickjs::Undefined)
                },
            )?;
            host_object.set("invokeFunc", host_invoke_func)?;
            host_object.set("invokeFunc0", host_invoke_func0)?;
        }
    Ok(())
}

fn build_var_object(this: rquickjs::Ctx<'_>) -> anyhow::Result<rquickjs::Object> {
        let var_set = rquickjs::Function::new(this.clone(), MutFn::new(move |cx: rquickjs::Ctx, args: rquickjs::prelude::Rest<rquickjs::Value<'_>>| {
            let var_name = args.first().ok_or_else(|| to_js_error(cx.clone(), anyhow!("Expected var_name argument")))?;
            let data = args.get(1).ok_or_else(||to_js_error(cx.clone(), anyhow!("Expected data argument")))?;
            if data.is_string() {
                let var_name_string = var_name.try_into_string()?.to_string()?;
                let data_string = var_name.try_into_string()?.to_string()?;
                var::set(var_name_string, data_string);
            } else if data.is_object() {
                let data = data.as_object().expect("Data should be an object");
                if data.is_array_buffer() {
                    let data = data.as_array_buffer().expect("Data should be an array buffer").as_bytes().ok_or_else(||to_js_error(cx.clone(), anyhow!("Could not get bytes from array buffer")))?;
                    let var_name_string = var_name.try_into_string()?.to_string()?;
                    var::set(var_name_string, data);
                }
            }
            Ok::<_, rquickjs::Error>(rquickjs::Undefined)
        }));
        let var_get = rquickjs::Function::new(this.clone(), |cx: rquickjs::Ctx, args: rquickjs::prelude::Rest<rquickjs::Value<'_>>| {
            let var_name = args.first().ok_or_else(|| to_js_error(cx.clone(), anyhow!("Expected var_name argument")))?.as_string().ok_or_else(|| to_js_error(cx.clone(), anyhow!("Expected var_name argument to be a string")))?.to_string()?;
            let data = var::get::<Vec<u8>>(var_name)?;
            match data {
                Some(d) => Ok(rquickjs::ArrayBuffer::new(cx.clone(), d)?.as_value()),
                None => Ok(&rquickjs::Null.into_value(cx.clone()))
            }
        })?;
        let var_get_str = rquickjs::Function::new(this.clone(), |cx: rquickjs::Ctx, args: rquickjs::prelude::Rest<rquickjs::Value<'_>>| {
            let var_name = args.first().ok_or_else(||to_js_error(cx.clone(), anyhow!("Expected var_name argument")))?.try_into_string()?.to_string()?;
            let data = var::get::<String>(var_name)?;
            match data {
                Some(d) => Ok(rquickjs::String::from_str(cx.clone(), &d)?.as_value()),
                None => Ok(&rquickjs::Null.into_value(cx.clone()))
            }
        })?;
        let var_object = rquickjs::Object::new(this.clone())?;
        var_object.set("set", var_set)?;
        var_object.set("getBytex", var_get)?;
        var_object.set("getString", var_get_str)?;
    Ok(var_object)
}

fn build_http_object<'js>(this: rquickjs::Ctx<'js>) -> anyhow::Result<rquickjs::Object> {
    let http_req = rquickjs::Function::new(this.clone(), |cx: rquickjs::Ctx<'js>, args: rquickjs::prelude::Rest<rquickjs::Value>| {
        let req = args.first().ok_or_else(|| to_js_error(cx.clone(), anyhow!("Expected http request argument")))?.clone();

        if !req.is_object() {
            return Err(to_js_error(cx, anyhow!("First argument should be an http request argument")));
        }

        let req = req.into_object().expect("Should be able to convert a request into an object");
        let url = req.get::<_, rquickjs::Value>("url").context("Http Request should have url property").map_err(|e| to_js_error(cx.clone(), e))?;

        let method_string = match req.get::<_, rquickjs::Value>("method") {
            Ok(m) => m.as_string().ok_or(rquickjs::Error::Unknown)?.to_string()?,
            Err(_) => "GET".to_string()
        };

        let mut http_req = HttpRequest::new(url.as_string().ok_or(rquickjs::Error::Unknown)?.to_string()?).with_method(method_string);

        let headers = req.get::<_, rquickjs::Value>("headers")?;

        if !headers.is_null() && !headers.is_undefined() {
            if !headers.is_object() {
                return Err(to_js_error(cx, anyhow!("Expected headers to be an object")));
            }
            let headers = headers.as_object().expect("Should be able to convert headers to an object");
            let header_values: rquickjs::object::ObjectIter<rquickjs::Value, rquickjs::Value> = headers.props();
            
            for property_result in header_values {
                let (key, value) = property_result?;
                let key = key.as_string().ok_or_else(|| to_js_error(cx.clone(), anyhow!("Expect headers keys to be a string")))?.to_string()?;
                let value = value.as_string().ok_or_else(|| to_js_error(cx.clone(), anyhow!("Header values should be able to be convertet to a string")))?.to_string()?;
                http_req.headers.insert(key, value);
            }
        }
        
        let body_args = args.get(1);
        let http_body = match body_args {
            None => None,
            Some(body) => {
                let body = body.as_string();
                if let Some(body_string) = body {
                    Some(body_string.to_string()?)
                } else {
                    None
                }
             }
        };

        let res = http::request(&http_req, http_body).map_err(|e| to_js_error(cx.clone(), e))?;
        let body = res.body();
        let body = from_utf8(&body).map_err(|e| to_js_error(cx.clone(), anyhow::Error::from(e)))?;

        let resp_obj = rquickjs::Object::new(cx.clone())?;

        resp_obj.set("body", rquickjs::String::from_str(cx.clone(), body)?.as_value().clone());

        resp_obj.set("status", rquickjs::Value::new_int(cx.clone(), res.status_code() as i32));
        Ok(resp_obj)
        })?; 

    let http_obj = rquickjs::Object::new(this.clone())?;
    http_obj.set("request", http_req)?;

    Ok(http_obj)
}

fn build_config_object<'js>(this: rquickjs::Ctx<'js>) -> anyhow::Result<rquickjs::Object<'js>> {
    let config_get = rquickjs::Function::new(this.clone(),  move |cx: rquickjs::Ctx<'js>, args: rquickjs::prelude::Rest<rquickjs::Value<'js>>| -> Result<_, rquickjs::Error > {
        let key = args.first().ok_or_else(|| to_js_error(cx.clone(), anyhow!("Expected key argument")))?;

        if !key.is_string() {
            to_js_error(cx.clone(), anyhow!("Expected key to be a string"));
        }

        let key = key.as_string().expect("Should be able to cast the string to a string").to_string()?;

        let config_val = match config::get(&key) {
            Ok(Some(v)) => rquickjs::String::from_str(cx.clone(), &v)?.as_value().clone(),
            _ => rquickjs::Value::new_null(cx)
        };
        Ok(config_val)
    })?;
    let config_obj = rquickjs::Object::new(this.clone())?;

    config_obj.set("get", config_get)?;

    Ok(config_obj)
}

fn build_memory(this: rquickjs::Ctx) -> anyhow::Result<rquickjs::Object> {
    let memory_from_buffer = rquickjs::Function::new(this.clone(), |cx: rquickjs::Ctx, args: rquickjs::prelude::Rest<rquickjs::Value>| {
        let data = args.first().ok_or_else(|| to_js_error(cx.clone(), anyhow!("Expected data argument")))?;
        if !data.is_object() {
            return Err(to_js_error(cx, anyhow!("Expected data to be an array buffer")))
        }
        let data = data.as_object().expect("Should be able to cast data as an object");
        if !data.is_array_buffer() {
            return Err(to_js_error(cx, anyhow!("Expected data to be an array buffer")))
        }
        let data = data.as_array_buffer().expect("Should be able to cast data as an array buffer").as_bytes().ok_or_else(|| to_js_error(cx, anyhow!("Problem getting data from the array buffer")))?;
        let m = extism_pdk::Memory::from_bytes(data).map_err(|e| to_js_error(cx, e))?;

        let mut mem = rquickjs::Object::new(cx.clone())?;

            
            let offset = rquickjs::BigInt::from_u64(cx.clone(), m.offset())?;
            let len = rquickjs::BigInt::from_u64(cx.clone(), m.len() as u64);
            mem.set("offset", offset)?;
            mem.set("len", len)?;
            Ok(mem)
    })?;
    let memory_find = rquickjs::Function::new(this.clone(), |cx, args: rquickjs::prelude::Rest<rquickjs::Value>| {
        let ptr = args.first().ok_or_else(|| to_js_error(cx, anyhow!("Expected offset argument")))?;
        if !ptr.is_number() {
            return Err(to_js_error(cx, anyhow!("Expected offset to be a number")))
        }
        let ptr = if ptr.is_int() {
            ptr.as_int().expect("Should be able to cast offset to int") as i64
        } else {
            ptr.as_number().expect("Should be able to cast offset to number") as i64
        };
        let Some(m) = extism_pdk::Memory::find(ptr as u64) else {
            return Err(to_js_error(cx, anyhow!("Offset did not represent a valid block of memory (offset={ptr:x})")))
        };
        let mem = rquickjs::Object::new(cx.clone())?;
        let offset = rquickjs::BigInt::from_u64(cx.clone(), m.offset())?;
        let len = rquickjs::BigInt::from_u64(cx.clone(), m.len() as u64);

        mem.set("offset", offset)?;
        mem.set("len", len)?;

        Ok(mem)
    })?;
    let memory_free = context.wrap_callback(
        |_ctx: &JSContextRef, _this: JSValueRef, args: &[JSValueRef]| {
            let ptr = args.first().ok_or(anyhow!("Expected offset argument"))?;
            if !ptr.is_number() {
                bail!("Expected an offset");
            }
            let ptr = if ptr.is_repr_as_i32() {
                ptr.as_i32_unchecked() as i64
            } else {
                ptr.as_f64_unchecked() as i64
            };
            if let Some(x) = extism_pdk::Memory::find(ptr as u64) {
                x.free();
            }

            Ok(JSValue::Undefined)
        },
    )?;
    let read_bytes = context.wrap_callback(
        |_ctx: &JSContextRef, _this: JSValueRef, args: &[JSValueRef]| {
            let ptr = args.first().ok_or(anyhow!("Expected offset argument"))?;
            if !ptr.is_number() {
                bail!("Expected an offset");
            }
            let ptr = if ptr.is_repr_as_i32() {
                ptr.as_i32_unchecked() as i64
            } else {
                ptr.as_f64_unchecked() as i64
            };
            let Some(m) = extism_pdk::Memory::find(ptr as u64) else {
                bail!("Offset did not represent a valid block of memory (offset={ptr:x})");
            };
            let bytes = m.to_vec();
            Ok(JSValue::ArrayBuffer(bytes))
        },
    )?;

    let mem_obj = context.object_value()?;
    mem_obj.set_property("_fromBuffer", memory_from_buffer)?;
    mem_obj.set_property("_find", memory_find)?;
    mem_obj.set_property("_free", memory_free)?;
    mem_obj.set_property("_readBytes", read_bytes)?;

    Ok(mem_obj)
}

fn build_clock(context: &JSContextRef) -> anyhow::Result<JSValueRef> {
    context.wrap_callback(get_time())
}

fn build_decoder(context: &JSContextRef) -> anyhow::Result<JSValueRef> {
    context.wrap_callback(decode_utf8_buffer_to_js_string())
}

fn build_encoder(context: &JSContextRef) -> anyhow::Result<JSValueRef> {
    context.wrap_callback(encode_js_string_to_utf8_buffer())
}

fn get_time() -> impl FnMut(&JSContextRef, JSValueRef, &[JSValueRef]) -> anyhow::Result<JSValue> {
    move |_ctx: &JSContextRef, _this: JSValueRef, _args: &[JSValueRef]| {
        let now = Utc::now();
        // This format is compatible with JavaScript's Date constructor
        let formatted = now.to_rfc3339_opts(SecondsFormat::Millis, true);
        Ok(formatted.into())
    }
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
