use anyhow::anyhow;
use once_cell::sync::OnceCell;
use quickjs_wasm_rs::{JSContextRef, JSValue, JSValueRef};
use rquickjs::function::Args;
use rquickjs::object::ObjectKeysIter;
use rquickjs::{qjs::JSValue as RawValue, Context, Runtime, Value};
use rquickjs::{Ctx, FromJs, Object};
use std::io;
use std::io::Read;

mod globals;

static mut CONTEXT: OnceCell<rquickjs::Context> = OnceCell::new();
static mut CALL_ARGS: Vec<Vec<ArgType>> = vec![];

#[export_name = "wizer.initialize"]
extern "C" fn init() {
    let runtime = Runtime::new().unwrap();
    let context = Context::full(&runtime).unwrap();
    globals::inject_globals(&context).expect("Failed to initialize globals");

    let mut code = String::new();
    io::stdin().read_to_string(&mut code).unwrap();

    let _ = context.with(|this| {
        this.eval(code)?;
        Ok::<_, rquickjs::Error>(rquickjs::Undefined)
    });

    unsafe {
        CONTEXT
            .set(context)
            .map_err(|_| anyhow::anyhow!("Could not intialize JS Context"))
            .unwrap()
    }
}

fn js_context() -> &'static rquickjs::Context {
    unsafe {
        if CONTEXT.get().is_none() {
            init()
        }

        let context = CONTEXT.get_unchecked();
        context
    }
}

fn convert_js_value<'js>(ctx: rquickjs::Ctx<'js>, arg: &ArgType) -> Value<'js> {
    let v = match arg {
        ArgType::I32(v) => rquickjs::Value::new_int(ctx, *v),
        ArgType::I64(v) => rquickjs::BigInt::from_i64(ctx, *v).unwrap().into(),
        ArgType::F32(v) => rquickjs::Value::new_float(ctx, *v as f64),
        ArgType::F64(v) => rquickjs::Value::new_float(ctx, *v),
    };

    v
}

fn invoke<'a, T, F: for<'b> Fn(Ctx<'b>, rquickjs::Value<'b>) -> T>(
    idx: i32,
    conv: F,
) -> Result<T, anyhow::Error> {
    let call_args = unsafe { CALL_ARGS.pop() };
    let context = js_context();
    let result = context.with(|ctx| {
        let call_args = call_args.unwrap();
        let args: Args = call_args.iter().fold(
            Args::new(ctx.clone(), call_args.len()),
            |mut args, rust_arg| {
                match rust_arg {
                    ArgType::I32(v) => args
                        .push_arg(v)
                        .expect("Should be able to convert i32 to JS arg"),
                    ArgType::I64(v) => args
                        .push_arg(v)
                        .expect("Should be able to convert i64 to JS arg"),
                    ArgType::F32(v) => args
                        .push_arg(v)
                        .expect("Should be able to convert f32 to JS arg"),
                    ArgType::F64(v) => args
                        .push_arg(v)
                        .expect("Should be able to convert f64 to JS arg"),
                }
                args
            },
        );

        let global = ctx.globals();

        let module: Object = global.get("module")?;
        let exports: Object = module.get("exports")?;

        let export_names = export_names(exports.clone()).unwrap();

        let function: rquickjs::Function =
            exports.get(export_names[idx as usize].as_str()).unwrap();

        let function_invocation_result = function.call_arg(args);

        match function_invocation_result {
            Ok(r) => {
                let res = conv(ctx.clone(), r);
                Ok(res)
            }
            Err(err) => {
                let e = format!("{:?}", err);
                let mem = extism_pdk::Memory::from_bytes(&e).unwrap();
                unsafe {
                    extism_pdk::extism::error_set(mem.offset());
                }
                Err(err)
            }
        }
    })?;
    Ok(result)
}

#[no_mangle]
pub extern "C" fn __arg_start() {
    unsafe {
        CALL_ARGS.push(vec![]);
    }
}

#[no_mangle]
pub extern "C" fn __arg_i32(arg: i32) {
    unsafe {
        CALL_ARGS.last_mut().unwrap().push(ArgType::I32(arg));
    }
}

#[no_mangle]
pub extern "C" fn __arg_i64(arg: i64) {
    unsafe {
        CALL_ARGS.last_mut().unwrap().push(ArgType::I64(arg));
    }
}

#[no_mangle]
pub extern "C" fn __arg_f32(arg: f32) {
    unsafe {
        CALL_ARGS.last_mut().unwrap().push(ArgType::F32(arg));
    }
}

#[no_mangle]
pub extern "C" fn __arg_f64(arg: f64) {
    unsafe {
        CALL_ARGS.last_mut().unwrap().push(ArgType::F64(arg));
    }
}

macro_rules! unwrap_value {
    ($d:expr, $x:expr) => {
        match $x {
            Ok(x) => x,
            Err(e) => {
                let err = format!("{:?}", e);
                let mem = extism_pdk::Memory::from_bytes(&err).unwrap();
                unsafe {
                    extism_pdk::extism::error_set(mem.offset());
                }
                $d
            }
        }
    };
}

#[no_mangle]
pub extern "C" fn __invoke_i32(idx: i32) -> i32 {
    unwrap_value!(-1, invoke(idx, |_ctx, r| r.as_number().unwrap() as i32))
}

#[no_mangle]
pub extern "C" fn __invoke_i64(idx: i32) -> i64 {
    unwrap_value!(
        -1,
        invoke(idx, |_ctx, r| (r.as_big_int().unwrap())
            .clone()
            .to_i64()
            .unwrap())
    )
}

#[no_mangle]
pub extern "C" fn __invoke_f64(idx: i32) -> f64 {
    unwrap_value!(-1.0, invoke(idx, |_ctx, r| r.as_float().unwrap()))
}

#[no_mangle]
pub extern "C" fn __invoke_f32(idx: i32) -> f32 {
    unwrap_value!(-1.0, invoke(idx, |_ctx, r| r.as_number().unwrap() as f32))
}

#[no_mangle]
pub extern "C" fn __invoke(idx: i32) {
    unwrap_value!((), invoke(idx, |_ctx, _r| ()))
}

fn export_names(exports: rquickjs::Object) -> anyhow::Result<Vec<String>> {
    let mut keys_iter: ObjectKeysIter<String> = exports.keys();
    let mut key = keys_iter.next();
    let mut keys: Vec<String> = vec![];
    while key.is_some() {
        keys.push(key.unwrap()?.to_string());
        key = keys_iter.next();
    }
    keys.sort();
    Ok(keys)
}

enum ArgType {
    I32(i32),
    I64(i64),
    F32(f32),
    F64(f64),
}
