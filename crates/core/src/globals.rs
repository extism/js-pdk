use std::{borrow::Cow, str::from_utf8};

use anyhow::{anyhow, Context};
use chrono::{SecondsFormat, Utc};
use extism_pdk::extism::load_input;
use extism_pdk::*;
use rquickjs::{
    function::MutFn, object, prelude::*, ArrayBuffer, BigInt, Context as JSContext, Ctx, FromJs,
    Function, IntoJs, Null, Object, Undefined, Value,
};

static PRELUDE: &[u8] = include_bytes!("prelude/dist/index.js"); // if this panics, run `make` from the root

pub fn inject_globals(context: &JSContext) -> anyhow::Result<()> {
    context.with(|this| {
        let module = build_module_object(this.clone()).map_err(|e| to_js_error(this.clone(), e))?;

        let console =
            build_console_object(this.clone()).map_err(|e| to_js_error(this.clone(), e))?;
        let var = build_var_object(this.clone()).map_err(|e| to_js_error(this.clone(), e))?;
        let http = build_http_object(this.clone()).map_err(|e| to_js_error(this.clone(), e))?;
        let cfg = build_config_object(this.clone()).map_err(|e| to_js_error(this.clone(), e))?;
        let decoder = build_decoder(this.clone())?;
        let encoder = build_encoder(this.clone())?;
        let clock = build_clock(this.clone())?;
        let mem = build_memory(this.clone()).map_err(|e| to_js_error(this.clone(), e))?;
        let host = build_host_object(this.clone()).map_err(|e| to_js_error(this.clone(), e))?;
        let global = this.globals();
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

        add_host_functions(this.clone()).map_err(|e| to_js_error(this.clone(), e))?;

        this.eval("globalThis.module = {}; globalThis.module.exports = {}")?;
        // need a *global* var for polyfills to work
        this.eval("var global = globalThis")?;
        this.eval(from_utf8(PRELUDE).map_err(|e| rquickjs::Error::Utf8(e))?)?;

        Ok::<_, rquickjs::Error>(())
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

fn get_args_as_str(args: &Rest<Value>) -> anyhow::Result<String> {
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

fn to_js_error(cx: Ctx, e: anyhow::Error) -> rquickjs::Error {
    match e.downcast::<rquickjs::Error>() {
        Ok(e) => e,
        Err(e) => cx.throw(Value::from_exception(
            rquickjs::Exception::from_message(cx.clone(), &e.to_string())
                .expect("Creating an exception should succeed"),
        )),
    }
}

fn build_console_object(this: Ctx) -> anyhow::Result<Object> {
    let console = Object::new(this.clone())?;
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

fn build_module_object(this: Ctx) -> anyhow::Result<Object> {
    let exports = Object::new(this.clone())?;
    let module = Object::new(this.clone())?;
    module.set("exports", exports)?;

    Ok(module)
}

fn build_host_object<'js>(this: Ctx<'js>) -> anyhow::Result<Object> {
    let host_input_bytes = Function::new(
        this.clone(),
        MutFn::new(move |cx| {
            let input = unsafe { load_input() };
            Ok::<_, rquickjs::Error>(ArrayBuffer::new(cx, input))
        }),
    )?;
    let host_input_string = Function::new(
        this.clone(),
        MutFn::new(move |cx| {
            let input = unsafe { load_input() };
            let input_string = String::from_utf8(input)?;
            rquickjs::String::from_str(cx, &input_string)
        }),
    )?;
    let host_output_bytes = Function::new(
        this.clone(),
        MutFn::new(move |cx: Ctx<'js>, args: Rest<Value<'js>>| {
            let output = args.first().unwrap().clone();
            let output_buffer = ArrayBuffer::from_value(output).unwrap();
            extism_pdk::output(output_buffer.as_bytes()).map_err(|e| to_js_error(cx.clone(), e))?;
            Ok::<_, rquickjs::Error>(Value::new_bool(cx, true))
        }),
    )?;
    let host_output_string = Function::new(
        this.clone(),
        MutFn::new(move |cx: Ctx<'js>, args: Rest<Value<'js>>| {
            let output = args.first().unwrap().clone();
            let output_string = output
                .as_string()
                .ok_or(rquickjs::Error::Unknown)?
                .to_string()?;
            extism_pdk::output(output_string).map_err(|e| to_js_error(cx.clone(), e))?;
            Ok::<_, rquickjs::Error>(Value::new_bool(cx, true))
        }),
    )?;
    let host_object = Object::new(this.clone())?;
    host_object.set("inputBytes", host_input_bytes)?;
    host_object.set("inputString", host_input_string)?;
    host_object.set("outputBytes", host_output_bytes)?;
    host_object.set("outputString", host_output_string)?;
    Ok(host_object)
}

fn add_host_functions(this: Ctx<'_>) -> anyhow::Result<()> {
    let globals = this.globals();
    let host_object = globals.get::<_, Object>("Host")?;
    let invoke_host = host_object.get::<_, Value>("invokeHost")?;
    if invoke_host.is_null() || invoke_host.is_undefined() {
        let host_invoke_func = Function::new(this.clone(), move |cx, args: Rest<Value<'_>>| {
            let func_id = args.first().unwrap().as_int().unwrap() as u32;
            let len = args.len() - 1;
            match len {
                0 => {
                    let result = unsafe { __invokeHostFunc_0_1(func_id) };

                    Ok(Value::new_float(cx, result as f64))
                }
                1 => {
                    let ptr = args.get(1).unwrap().as_float().unwrap();
                    let result = unsafe { __invokeHostFunc_1_1(func_id, ptr as u64) };
                    Ok(Value::new_float(cx, result as f64))
                }
                2 => {
                    let ptr = args.get(1).unwrap().as_float().unwrap();
                    let ptr2 = args.get(2).unwrap().as_float().unwrap();
                    let result = unsafe { __invokeHostFunc_2_1(func_id, ptr as u64, ptr2 as u64) };
                    Ok(Value::new_float(cx, result as f64))
                }
                3 => {
                    let ptr = args.get(1).unwrap().as_float().unwrap();
                    let ptr2 = args.get(2).unwrap().as_float().unwrap();
                    let ptr3 = args.get(3).unwrap().as_float().unwrap();
                    let result = unsafe {
                        __invokeHostFunc_3_1(func_id, ptr as u64, ptr2 as u64, ptr3 as u64)
                    };
                    Ok(Value::new_float(cx, result as f64))
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
                    Ok(Value::new_float(cx, result as f64))
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
                    Ok(Value::new_float(cx, result as f64))
                }
                n => Err(to_js_error(
                    cx,
                    anyhow!("__invokeHostFunc with {n} parameters is not implemented"),
                )),
            }
        })?;
        let host_invoke_func0 = Function::new(this.clone(), move |cx: Ctx, args: Rest<Value>| {
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
                    unsafe { __invokeHostFunc_3_0(func_id, ptr as u64, ptr2 as u64, ptr3 as u64) };
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
            Ok(Undefined)
        })?;
        host_object.set("invokeFunc", host_invoke_func)?;
        host_object.set("invokeFunc0", host_invoke_func0)?;
    }
    Ok(())
}

fn build_var_object<'js>(this: Ctx<'js>) -> anyhow::Result<Object> {
    let var_set = Function::new(
        this.clone(),
        MutFn::new(move |cx: Ctx, args: Rest<Value<'_>>| {
            let var_name = args
                .first()
                .ok_or_else(|| to_js_error(cx.clone(), anyhow!("Expected var_name argument")))?;
            let data = args
                .get(1)
                .ok_or_else(|| to_js_error(cx.clone(), anyhow!("Expected data argument")))?;
            if data.is_string() {
                let var_name_string = var_name
                    .as_string()
                    .ok_or_else(|| {
                        to_js_error(
                            cx.clone(),
                            anyhow!("Expected var_name value to be a string"),
                        )
                    })?
                    .to_string()?;
                let data_string = data
                    .as_string()
                    .expect(
                        "Should be able to convert data to string since data.is_string() is true",
                    )
                    .to_string()?;
                var::set(var_name_string, data_string).map_err(|e| to_js_error(cx.clone(), e))?;
            } else if data.is_object() {
                let data = data.as_object().expect("Data should be an object");
                if data.is_array_buffer() {
                    let data = data
                        .as_array_buffer()
                        .expect("Data should be an array buffer")
                        .as_bytes()
                        .ok_or_else(|| {
                            to_js_error(
                                cx.clone(),
                                anyhow!("Could not get bytes from array buffer"),
                            )
                        })?;
                    let var_name_string = var_name
                        .as_string()
                        .ok_or_else(|| {
                            to_js_error(cx.clone(), anyhow!("Expected var_name arg to be a string"))
                        })?
                        .to_string()?;
                    var::set(var_name_string, data).map_err(|e| to_js_error(cx.clone(), e))?;
                }
            }
            Ok::<_, rquickjs::Error>(Undefined)
        }),
    );
    let var_get = Function::new(
        this.clone(),
        |cx: Ctx<'js>, args: Rest<Value<'js>>| -> Result<Value<'js>, rquickjs::Error> {
            let var_name = args
                .first()
                .ok_or_else(|| to_js_error(cx.clone(), anyhow!("Expected var_name argument")))?
                .as_string()
                .ok_or_else(|| {
                    to_js_error(
                        cx.clone(),
                        anyhow!("Expected var_name argument to be a string"),
                    )
                })?
                .to_string()?;
            let data = var::get::<Vec<u8>>(var_name).map_err(|e| to_js_error(cx.clone(), e))?;
            match data {
                Some(d) => {
                    let buffer = ArrayBuffer::new(cx.clone(), d)?;
                    Ok(buffer.as_value().clone())
                }
                None => Ok(Null.into_value(cx.clone())),
            }
        },
    )?;
    let var_get_str = Function::new(
        this.clone(),
        |cx: Ctx<'js>, args: Rest<Value<'_>>| -> Result<Value<'js>, rquickjs::Error> {
            let var_name = args
                .first()
                .ok_or_else(|| to_js_error(cx.clone(), anyhow!("Expected var_name argument")))?
                .as_string()
                .ok_or_else(|| {
                    to_js_error(
                        cx.clone(),
                        anyhow!("Expected var_name argument to be a string"),
                    )
                })?
                .to_string()?;
            let data = var::get::<String>(var_name).map_err(|e| to_js_error(cx.clone(), e))?;
            match data {
                Some(d) => {
                    let s = rquickjs::String::from_str(cx.clone(), &d)?;
                    Ok(s.as_value().clone())
                }
                None => Ok(Null.into_value(cx.clone())),
            }
        },
    )?;
    let var_object = Object::new(this.clone())?;
    var_object.set("set", var_set)?;
    var_object.set("getBytex", var_get)?;
    var_object.set("getString", var_get_str)?;
    Ok(var_object)
}

fn build_http_object<'js>(this: Ctx<'js>) -> anyhow::Result<Object> {
    let http_req = Function::new(this.clone(), |cx: Ctx<'js>, args: Rest<Value>| {
        let req = args
            .first()
            .ok_or_else(|| to_js_error(cx.clone(), anyhow!("Expected http request argument")))?
            .clone();

        if !req.is_object() {
            return Err(to_js_error(
                cx,
                anyhow!("First argument should be an http request argument"),
            ));
        }

        let req = req
            .into_object()
            .expect("Should be able to convert a request into an object");
        let url = req
            .get::<_, Value>("url")
            .context("Http Request should have url property")
            .map_err(|e| to_js_error(cx.clone(), e))?;

        let method_string = match req.get::<_, Value>("method") {
            Ok(m) => m.as_string().ok_or(rquickjs::Error::Unknown)?.to_string()?,
            Err(_) => "GET".to_string(),
        };

        let mut http_req = HttpRequest::new(
            url.as_string()
                .ok_or(rquickjs::Error::Unknown)?
                .to_string()?,
        )
        .with_method(method_string);

        let headers = req.get::<_, Value>("headers")?;

        if !headers.is_null() && !headers.is_undefined() {
            if !headers.is_object() {
                return Err(to_js_error(cx, anyhow!("Expected headers to be an object")));
            }
            let headers = headers
                .as_object()
                .expect("Should be able to convert headers to an object");
            let header_values: object::ObjectIter<Value, Value> = headers.props();

            for property_result in header_values {
                let (key, value) = property_result?;
                let key = key
                    .as_string()
                    .ok_or_else(|| {
                        to_js_error(cx.clone(), anyhow!("Expect headers keys to be a string"))
                    })?
                    .to_string()?;
                let value = value
                    .as_string()
                    .ok_or_else(|| {
                        to_js_error(
                            cx.clone(),
                            anyhow!("Header values should be able to be convertet to a string"),
                        )
                    })?
                    .to_string()?;
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

        let resp_obj = Object::new(cx.clone())?;

        resp_obj.set(
            "body",
            rquickjs::String::from_str(cx.clone(), body)?
                .as_value()
                .clone(),
        )?;

        resp_obj.set(
            "status",
            Value::new_int(cx.clone(), res.status_code() as i32),
        )?;
        Ok(resp_obj)
    })?;

    let http_obj = Object::new(this.clone())?;
    http_obj.set("request", http_req)?;

    Ok(http_obj)
}

fn build_config_object<'js>(this: Ctx<'js>) -> anyhow::Result<Object<'js>> {
    let config_get = Function::new(
        this.clone(),
        move |cx: Ctx<'js>, args: Rest<Value<'js>>| -> Result<_, rquickjs::Error> {
            let key = args
                .first()
                .ok_or_else(|| to_js_error(cx.clone(), anyhow!("Expected key argument")))?;

            if !key.is_string() {
                to_js_error(cx.clone(), anyhow!("Expected key to be a string"));
            }

            let key = key
                .as_string()
                .expect("Should be able to cast the string to a string")
                .to_string()?;

            let config_val = match config::get(&key) {
                Ok(Some(v)) => rquickjs::String::from_str(cx.clone(), &v)?
                    .as_value()
                    .clone(),
                _ => Value::new_null(cx),
            };
            Ok(config_val)
        },
    )?;
    let config_obj = Object::new(this.clone())?;

    config_obj.set("get", config_get)?;

    Ok(config_obj)
}

fn build_memory<'js>(this: Ctx<'js>) -> anyhow::Result<Object> {
    let memory_from_buffer = Function::new(this.clone(), |cx: Ctx<'js>, args: Rest<Value>| {
        let data = args
            .first()
            .ok_or_else(|| to_js_error(cx.clone(), anyhow!("Expected data argument")))?;
        if !data.is_object() {
            return Err(to_js_error(
                cx,
                anyhow!("Expected data to be an array buffer"),
            ));
        }
        let data = data
            .as_object()
            .expect("Should be able to cast data as an object");
        if !data.is_array_buffer() {
            return Err(to_js_error(
                cx,
                anyhow!("Expected data to be an array buffer"),
            ));
        }
        let data = data
            .as_array_buffer()
            .expect("Should be able to cast data as an array buffer")
            .as_bytes()
            .ok_or_else(|| {
                to_js_error(
                    cx.clone(),
                    anyhow!("Problem getting data from the array buffer"),
                )
            })?;
        let m = extism_pdk::Memory::from_bytes(data).map_err(|e| to_js_error(cx.clone(), e))?;

        let mem = Object::new(cx.clone())?;

        let offset = BigInt::from_u64(cx.clone(), m.offset())?;
        let len = BigInt::from_u64(cx.clone(), m.len() as u64);
        mem.set("offset", offset)?;
        mem.set("len", len)?;
        Ok(mem)
    })?;
    let memory_find = Function::new(
        this.clone(),
        |cx: Ctx<'js>, args: Rest<Value>| -> Result<Value, rquickjs::Error> {
            let ptr = args
                .first()
                .ok_or_else(|| to_js_error(cx.clone(), anyhow!("Expected offset argument")))?;
            if !ptr.is_number() {
                return Err(to_js_error(
                    cx.clone(),
                    anyhow!("Expected offset to be a number"),
                ));
            }
            let ptr = if ptr.is_int() {
                ptr.as_int().expect("Should be able to cast offset to int") as i64
            } else {
                ptr.as_number()
                    .expect("Should be able to cast offset to number") as i64
            };
            let Some(m) = extism_pdk::Memory::find(ptr as u64) else {
                return Ok(Undefined.into_value(cx.clone()));
            };
            let mem = Object::new(cx.clone())?;
            let offset = BigInt::from_u64(cx.clone(), m.offset())?;
            let len = BigInt::from_u64(cx.clone(), m.len() as u64);

            mem.set("offset", offset)?;
            mem.set("len", len)?;

            Ok(mem.into_value())
        },
    )?;
    let memory_free = Function::new(this.clone(), |cx: Ctx<'js>, args: Rest<Value>| {
        let ptr = args
            .first()
            .ok_or_else(|| to_js_error(cx.clone(), anyhow!("Expected offset argument")))?;
        if !ptr.is_number() {
            return Err(to_js_error(cx.clone(), anyhow!("Expected an offset")));
        }
        let ptr = if ptr.is_int() {
            ptr.as_int()
                .expect("Should be able to cast offset to an int") as i64
        } else {
            ptr.as_number()
                .expect("Should be able to cast offset to number") as i64
        };

        if let Some(x) = extism_pdk::Memory::find(ptr as u64) {
            x.free();
        }
        Ok(Undefined)
    })?;

    let read_bytes = Function::new(this.clone(), |cx: Ctx<'js>, args: Rest<Value>| {
        let ptr = args
            .first()
            .ok_or_else(|| to_js_error(cx.clone(), anyhow!("Expected offset argument")))?;

        if !ptr.is_number() {
            return Err(to_js_error(cx.clone(), anyhow!("Expected an offset")));
        }

        let ptr = if ptr.is_int() {
            ptr.as_int()
                .expect("Should be able to cast offset to an int") as i64
        } else {
            ptr.as_number()
                .expect("Should be able to cast offset to number") as i64
        };

        let Some(m) = extism_pdk::Memory::find(ptr as u64) else {
            return Err(to_js_error(
                cx.clone(),
                anyhow!("Offset did not represent a valid block of memory (offset={ptr:x})"),
            ));
        };

        let bytes = m.to_vec();

        Ok(ArrayBuffer::new(cx, bytes))
    })?;

    let mem_obj = Object::new(this.clone())?;

    mem_obj.set("_fromBuffer", memory_from_buffer)?;
    mem_obj.set("_find", memory_find)?;
    mem_obj.set("_free", memory_free)?;
    mem_obj.set("_readBytes", read_bytes)?;

    Ok(mem_obj)
}

fn build_clock(this: Ctx) -> rquickjs::Result<Function> {
    Function::new(this, get_time())
}

fn build_decoder(this: Ctx) -> rquickjs::Result<Function> {
    Function::new(this, decode_utf8_buffer_to_js_string())
}

fn build_encoder(this: Ctx) -> rquickjs::Result<Function> {
    Function::new(this, encode_js_string_to_utf8_buffer())
}

fn get_time<'js>(
) -> MutFn<impl Fn(Ctx<'js>, Rest<Value<'js>>) -> rquickjs::Result<rquickjs::String<'js>>> {
    MutFn::new(|cx: Ctx<'js>, _args| {
        let now = Utc::now();
        // This format is compatible with JavaScript's Date constructor
        let formatted = now.to_rfc3339_opts(SecondsFormat::Millis, true);
        Ok(rquickjs::String::from_str(cx.clone(), &formatted)?)
    })
}

fn decode_utf8_buffer_to_js_string<'js>(
) -> MutFn<impl Fn(Ctx<'js>, Rest<Value<'js>>) -> rquickjs::Result<Value<'js>>> {
    MutFn::new(move |cx: Ctx<'js>, args: Rest<Value<'js>>| {
        if args.len() != 5 {
            return Err(to_js_error(
                cx,
                anyhow!("Expecting 5 arguments, received {}", args.len()),
            ));
        }

        let buffer: Vec<u8> = Vec::from_js(
            &cx,
            args.first().expect("Should have a first argument").clone(),
        )?;
        let byte_offset = args[1]
            .as_big_int()
            .expect("Should be able to bet byte_offset as int")
            .clone()
            .to_i64()? as usize;
        let byte_length = args[2]
            .as_big_int()
            .expect("Should be able to cast byte_length as int")
            .clone()
            .to_i64()? as usize;
        let fatal = args[3]
            .as_bool()
            .expect("Should be able to cast fatal as bool");
        let ignore_bom = args[4]
            .as_bool()
            .expect("Should be able to cast ignore_bom as bool");

        let mut view = buffer
            .get(byte_offset..(byte_offset + byte_length))
            .ok_or_else(|| anyhow!("Provided offset and length is not valid for provided buffer"))
            .map_err(|e| to_js_error(cx.clone(), e))?;

        if !ignore_bom {
            view = match view {
                // [0xEF, 0xBB, 0xBF] is the UTF-8 BOM which we want to strip
                [0xEF, 0xBB, 0xBF, rest @ ..] => rest,
                _ => view,
            }
        }

        let str = if fatal {
            Cow::from(from_utf8(view).map_err(|e| rquickjs::Error::Utf8(e))?)
        } else {
            String::from_utf8_lossy(view)
        };

        Ok(rquickjs::String::from_str(cx.clone(), &str.to_string())?.into())
    })
}

fn encode_js_string_to_utf8_buffer<'js>(
) -> MutFn<impl Fn(Ctx<'js>, Rest<Value<'js>>) -> rquickjs::Result<Value<'js>>> {
    MutFn::new(move |cx: Ctx<'js>, args: Rest<Value<'js>>| {
        if args.len() != 1 {
            return Err(to_js_error(
                cx,
                anyhow!("Expecting 1 argument, got {}", args.len()),
            ));
        }

        let js_string = args[0].clone();
        let rust_string = String::from_js(&cx, js_string)?;
        let buffer = rust_string.as_bytes();
        Vec::from_bytes(buffer).unwrap().into_js(&cx)
    })
}
