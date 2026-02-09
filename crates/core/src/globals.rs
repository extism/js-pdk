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

        let console_write = build_console_writer(this.clone())?;
        let var = build_var_object(this.clone()).map_err(|e| to_js_error(this.clone(), e))?;
        let http = build_http_object(this.clone()).map_err(|e| to_js_error(this.clone(), e))?;
        let cfg = build_config_object(this.clone()).map_err(|e| to_js_error(this.clone(), e))?;
        let decoder = build_decoder(this.clone())?;
        let encoder = build_encoder(this.clone())?;
        let clock = build_clock(this.clone())?;
        let clock_ms = build_clock_ms(this.clone())?;
        let random_bytes = build_random_bytes(this.clone())?;
        let sha_digest = build_sha_digest(this.clone())?;
        let mem = build_memory(this.clone()).map_err(|e| to_js_error(this.clone(), e))?;
        let host = build_host_object(this.clone()).map_err(|e| to_js_error(this.clone(), e))?;
        let global = this.globals();
        global.set("__consoleWrite", console_write)?;
        global.set("module", module)?;
        global.set("Host", host)?;
        global.set("Var", var)?;
        global.set("Http", http)?;
        global.set("Config", cfg)?;
        global.set("Memory", mem)?;
        global.set("__decodeUtf8BufferToString", decoder)?;
        global.set("__encodeStringToUtf8Buffer", encoder)?;
        global.set("__getTime", clock)?;
        global.set("__getTimeMs", clock_ms)?;
        global.set("__getRandomBytes", random_bytes)?;
        global.set("__shaDigest", sha_digest)?;

        add_host_functions(this.clone()).map_err(|e| to_js_error(this.clone(), e))?;

        this.eval::<(), _>("globalThis.module = {}; globalThis.module.exports = {}")?;
        // need a *global* var for polyfills to work
        this.eval::<(), _>("var global = globalThis")?;
        this.eval::<(), _>(from_utf8(PRELUDE).map_err(rquickjs::Error::Utf8)?)?;

        Ok::<_, rquickjs::Error>(())
    })?;

    Ok(())
}

#[link(wasm_import_module = "shim")]
extern "C" {
    fn __invokeHostFunc(
        func_idx: u32,
        arg0: u64,
        arg1: u64,
        arg2: u64,
        arg3: u64,
        arg4: u64,
    ) -> u64;
    fn __get_function_return_type(func_idx: u32) -> u32;
    fn __get_function_arg_type(func_idx: u32, arg_idx: u32) -> u32;
}

fn to_js_error(cx: Ctx, e: anyhow::Error) -> rquickjs::Error {
    match e.downcast::<rquickjs::Error>() {
        Ok(e) => e,
        Err(e) => cx.throw(rquickjs::Value::from_string(
            rquickjs::String::from_str(cx.clone(), &e.to_string())
                .expect("rquickjs error conversion"),
        )),
    }
}

fn build_console_writer<'js>(this: Ctx<'js>) -> Result<Function<'js>, rquickjs::Error> {
    Function::new(
        this.clone(),
        MutFn::new(move |cx: Ctx<'js>, args: Rest<Value<'js>>| {
            if args.len() != 2 {
                return Err(to_js_error(
                    cx.clone(),
                    anyhow!("Expected level and message arguments"),
                ));
            }

            let level = args[0]
                .as_string()
                .and_then(|s| s.to_string().ok())
                .ok_or_else(|| to_js_error(cx.clone(), anyhow!("Level must be a string")))?;
            let message = args[1]
                .as_string()
                .and_then(|s| s.to_string().ok())
                .ok_or_else(|| to_js_error(cx.clone(), anyhow!("Message must be a string")))?;

            match level.as_str() {
                "info" | "log" => info!("{}", message),
                "warn" => warn!("{}", message),
                "error" => error!("{}", message),
                "debug" => debug!("{}", message),
                "trace" => trace!("{}", message),
                _ => warn!("{}", message), // Default to warn for unknown levels, this should never happen
            }

            Ok(())
        }),
    )
}

fn build_module_object(this: Ctx) -> anyhow::Result<Object> {
    let exports = Object::new(this.clone())?;
    let module = Object::new(this.clone())?;
    module.set("exports", exports)?;

    Ok(module)
}

fn build_host_object<'js>(this: Ctx<'js>) -> anyhow::Result<Object<'js>> {
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

    let to_base64 = Function::new(
        this.clone(),
        MutFn::new(move |cx: Ctx<'js>, args: Rest<Value<'js>>| {
            let data = args.first().unwrap();

            let data = data
                .as_object()
                .ok_or_else(|| to_js_error(cx.clone(), anyhow!("Expected an ArrayBuffer")))?;

            if !data.is_array_buffer() {
                return Err(to_js_error(cx.clone(), anyhow!("Expected an ArrayBuffer")));
            }

            let bytes = data
                .as_array_buffer()
                .expect("Should be able to cast data as an array buffer")
                .as_bytes()
                .ok_or_else(|| {
                    to_js_error(cx.clone(), anyhow!("Could not get bytes from ArrayBuffer"))
                })?;

            use base64::prelude::*;
            let as_string = BASE64_STANDARD.encode(bytes);

            rquickjs::String::from_str(cx.clone(), &as_string)
        }),
    )?;

    let from_base64 = Function::new(
        this.clone(),
        MutFn::new(move |cx: Ctx<'js>, args: Rest<Value<'js>>| {
            let data = args.first().unwrap();
            if !data.is_string() {
                return Err(to_js_error(cx.clone(), anyhow!("expected string")));
            }

            use base64::prelude::*;
            let string = data.as_string().ok_or_else(|| {
                to_js_error(cx.clone(), anyhow!("Could not convert value into string"))
            })?;
            let string = string.to_string()?;
            let bytes = BASE64_STANDARD
                .decode(string)
                .map_err(|e| to_js_error(cx.clone(), e.into()))?;
            ArrayBuffer::new(cx.clone(), bytes)
        }),
    )?;

    let host_object = Object::new(this.clone())?;
    host_object.set("inputBytes", host_input_bytes)?;
    host_object.set("inputString", host_input_string)?;
    host_object.set("outputBytes", host_output_bytes)?;
    host_object.set("outputString", host_output_string)?;
    host_object.set("arrayBufferToBase64", to_base64)?;
    host_object.set("base64ToArrayBuffer", from_base64)?;
    Ok(host_object)
}

fn add_host_functions<'a>(this: Ctx<'a>) -> anyhow::Result<()> {
    let globals = this.globals();
    let host_object = globals.get::<_, Object>("Host")?;

    let host_invoke = Function::new(
        this.clone(),
        move |cx: Ctx<'a>, args: Rest<Value<'a>>| -> Result<Value<'a>, rquickjs::Error> {
            let func_id = args.first().unwrap().as_int().unwrap() as u32;
            let mut params = [0u64; 5];

            // Skip the first argument which is the function id
            // and convert the rest of the arguments to their 64-bit representation
            for i in 1..args.len() {
                let arg = args.get(i).unwrap();
                params[i - 1] = convert_to_u64_bits(arg, func_id, (i - 1) as u32);
            }

            let result = unsafe {
                __invokeHostFunc(
                    func_id, params[0], params[1], params[2], params[3], params[4],
                )
            };

            // Return the result as the appropriate JS value
            let return_type = unsafe { __get_function_return_type(func_id) };
            Ok(match return_type {
                TYPE_VOID => Undefined.into_value(cx.clone()),
                TYPE_I32 => Value::new_float(cx, (result & 0xFFFFFFFF) as i32 as f64),
                TYPE_I64 => Value::new_float(cx, result as f64),
                TYPE_F32 => Value::new_float(cx, f32::from_bits(result as u32) as f64),
                TYPE_F64 => Value::new_float(cx, f64::from_bits(result)),
                _ => panic!("Unsupported return type: {:?}", return_type),
            })
        },
    )?;

    host_object.set("invokeFunc", host_invoke)?;
    Ok(())
}

const TYPE_VOID: u32 = 0;
const TYPE_I32: u32 = 1;
const TYPE_I64: u32 = 2;
const TYPE_F32: u32 = 3;
const TYPE_F64: u32 = 4;

fn convert_to_u64_bits(value: &Value, func_id: u32, arg_idx: u32) -> u64 {
    match unsafe { __get_function_arg_type(func_id, arg_idx) } {
        TYPE_I32 => value.as_number().unwrap_or_default() as i32 as u64,
        TYPE_I64 => value
            .as_big_int()
            .and_then(|b| b.clone().to_i64().ok())
            .or_else(|| value.as_number().map(|n| n as i64))
            .unwrap_or_default() as u64,
        TYPE_F32 => {
            let f = value.as_number().unwrap_or_default() as f32;
            f.to_bits() as u64
        }
        TYPE_F64 => value
            .as_float()
            .or_else(|| value.as_number())
            .unwrap_or_default()
            .to_bits(),
        _ => panic!(
            "{}, {} unsupported type: {:?}",
            func_id,
            arg_idx,
            value.type_of()
        ),
    }
}


fn build_var_object<'js>(this: Ctx<'js>) -> anyhow::Result<Object<'js>> {
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
    var_object.set("getBytes", var_get)?;
    var_object.set("getString", var_get_str)?;
    Ok(var_object)
}

fn build_http_object<'js>(this: Ctx<'js>) -> anyhow::Result<Object<'js>> {
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

        let headers = rquickjs::Object::new(cx.clone())?;

        for (k, v) in res.headers() {
            headers.set(k, v)?;
        }

        resp_obj.set("headers", headers)?;
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

fn build_memory<'js>(this: Ctx<'js>) -> anyhow::Result<Object<'js>> {
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
            if !ptr.is_number() && ptr.type_of() != rquickjs::Type::BigInt {
                return Err(to_js_error(
                    cx.clone(),
                    anyhow!("Expected offset to be a number"),
                ));
            }
            let ptr = if ptr.is_int() {
                ptr.as_int().expect("Should be able to cast offset to int") as i64
            } else if ptr.type_of() == rquickjs::Type::BigInt {
                ptr.clone()
                    .try_into_big_int()
                    .expect("Should be able to cast offset to big int if it's type_of == BigInt")
                    .to_i64()
                    .expect("Should be able to cast BigInto to i64")
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
        if !ptr.is_number() && ptr.type_of() != rquickjs::Type::BigInt {
            return Err(to_js_error(cx.clone(), anyhow!("Expected an offset")));
        }
        let ptr = if ptr.is_int() {
            ptr.as_int()
                .expect("Should be able to cast offset to an int") as i64
        } else if ptr.type_of() == rquickjs::Type::BigInt {
            ptr.clone()
                .try_into_big_int()
                .expect("Should be able to cast offset to big int if it's type_of == BigInt")
                .to_i64()
                .expect("Should be able to cast BigInto to i64")
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

        if !ptr.is_number() && ptr.type_of() != rquickjs::Type::BigInt {
            return Err(to_js_error(cx.clone(), anyhow!("Expected an offset")));
        }

        let ptr = if ptr.is_int() {
            ptr.as_int()
                .expect("Should be able to cast offset to an int") as i64
        } else if ptr.type_of() == rquickjs::Type::BigInt {
            ptr.clone()
                .try_into_big_int()
                .expect("Should be able to cast offset to big int if it's type_of == BigInt")
                .to_i64()
                .expect("Should be able to cast BigInto to i64")
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

        ArrayBuffer::new(cx, bytes)
    })?;

    let mem_obj = Object::new(this.clone())?;

    mem_obj.set("_fromBuffer", memory_from_buffer)?;
    mem_obj.set("_find", memory_find)?;
    mem_obj.set("_free", memory_free)?;
    mem_obj.set("_readBytes", read_bytes)?;

    Ok(mem_obj)
}

fn build_random_bytes(this: Ctx) -> rquickjs::Result<Function> {
    Function::new(this, get_random_bytes())
}

fn build_sha_digest(this: Ctx) -> rquickjs::Result<Function> {
    Function::new(this, sha_digest())
}

fn sha_digest<'js>(
) -> MutFn<impl Fn(Ctx<'js>, Rest<Value<'js>>) -> rquickjs::Result<Value<'js>>> {
    MutFn::new(|cx: Ctx<'js>, args: Rest<Value<'js>>| {
        use sha2::Digest;

        let algo = args
            .first()
            .and_then(|v| v.as_string())
            .ok_or_else(|| to_js_error(cx.clone(), anyhow!("Expected algorithm name")))?
            .to_string()?;

        let data = args
            .get(1)
            .and_then(|v| ArrayBuffer::from_value(v.clone()))
            .ok_or_else(|| to_js_error(cx.clone(), anyhow!("Expected ArrayBuffer data")))?;

        let bytes = data
            .as_bytes()
            .ok_or_else(|| to_js_error(cx.clone(), anyhow!("Could not read ArrayBuffer")))?;

        let result: Vec<u8> = match algo.as_str() {
            "SHA-1" => {
                let mut hasher = sha1::Sha1::new();
                hasher.update(bytes);
                hasher.finalize().to_vec()
            }
            "SHA-256" => {
                let mut hasher = sha2::Sha256::new();
                hasher.update(bytes);
                hasher.finalize().to_vec()
            }
            "SHA-384" => {
                let mut hasher = sha2::Sha384::new();
                hasher.update(bytes);
                hasher.finalize().to_vec()
            }
            "SHA-512" => {
                let mut hasher = sha2::Sha512::new();
                hasher.update(bytes);
                hasher.finalize().to_vec()
            }
            _ => {
                return Err(to_js_error(
                    cx,
                    anyhow!("Unsupported algorithm: {}", algo),
                ))
            }
        };

        Ok(ArrayBuffer::new(cx, result)?.into_value())
    })
}

fn get_random_bytes<'js>(
) -> MutFn<impl Fn(Ctx<'js>, Rest<Value<'js>>) -> rquickjs::Result<Value<'js>>> {
    MutFn::new(|cx: Ctx<'js>, args: Rest<Value<'js>>| {
        let n = args
            .first()
            .and_then(|v| v.as_number())
            .ok_or_else(|| to_js_error(cx.clone(), anyhow!("Expected byte count argument")))?
            as usize;
        let mut buf = vec![0u8; n];
        getrandom::getrandom(&mut buf)
            .map_err(|e| to_js_error(cx.clone(), anyhow!("getrandom failed: {}", e)))?;
        Ok(ArrayBuffer::new(cx, buf)?.into_value())
    })
}

fn build_clock(this: Ctx) -> rquickjs::Result<Function> {
    Function::new(this, get_time())
}

fn build_clock_ms(this: Ctx) -> rquickjs::Result<Function> {
    Function::new(this, get_time_ms())
}

fn get_time_ms<'js>(
) -> MutFn<impl Fn(Ctx<'js>, Rest<Value<'js>>) -> rquickjs::Result<Value<'js>>> {
    MutFn::new(|cx: Ctx<'js>, _args| {
        let now = Utc::now();
        Ok(Value::new_float(cx, now.timestamp_millis() as f64))
    })
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
        rquickjs::String::from_str(cx.clone(), &formatted)
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

        let js_buffer_value = args.first().expect("Should have a first argument").clone();
        let buffer: Vec<u8> = if js_buffer_value.is_array() {
            Vec::from_js(&cx, js_buffer_value)?
        } else {
            let Some(array_buffer) = ArrayBuffer::from_value(js_buffer_value) else {
                Err(to_js_error(
                    cx.clone(),
                    anyhow!("Could not cast the buffer arg to an ArrayBuffer"),
                ))?
            };
            let Some(bytes) = array_buffer.as_bytes() else {
                Err(to_js_error(
                    cx.clone(),
                    anyhow!("Could not get the bytes from the array_buffer"),
                ))?
            };
            Vec::from_bytes(bytes).map_err(|e| to_js_error(cx.clone(), e))?
        };
        let byte_offset = args[1].as_number().unwrap_or_default() as usize;
        let byte_length = args[2].as_number().unwrap_or_default() as usize;
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
            Cow::from(from_utf8(view).map_err(rquickjs::Error::Utf8)?)
        } else {
            String::from_utf8_lossy(view)
        };

        Ok(rquickjs::String::from_str(cx.clone(), &str)?.into())
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