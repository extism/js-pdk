use once_cell::sync::OnceCell;
use rquickjs::{
    function::Args, object::ObjectKeysIter, Context, Ctx, Function, Object, Runtime, Undefined,
    Value,
};
use std::io;
use std::io::Read;

mod globals;

static mut CONTEXT: OnceCell<Context> = OnceCell::new();
static mut CALL_ARGS: Vec<Vec<ArgType>> = vec![];

#[export_name = "wizer.initialize"]
extern "C" fn init() {
    let runtime = Runtime::new().expect("Couldn't make a runtime");
    let context = Context::full(&runtime).expect("Couldnt make a context");
    globals::inject_globals(&context).expect("Failed to initialize globals");

    let mut code = String::new();
    io::stdin().read_to_string(&mut code).unwrap();

    let _ = context.with(|this| -> Result<rquickjs::Undefined, rquickjs::Error> {
        match this.eval(code) {
            Ok(()) => (),
            Err(e) => return Err(e),
        }
        Ok(Undefined)
    });

    unsafe {
        CONTEXT
            .set(context)
            .map_err(|_| anyhow::anyhow!("Could not intialize JS Context"))
            .unwrap()
    }
}

fn js_context() -> &'static Context {
    unsafe {
        if CONTEXT.get().is_none() {
            init()
        }
        let context = CONTEXT.get_unchecked();
        context
    }
}

fn invoke<'a, T, F: for<'b> Fn(Ctx<'b>, Value<'b>) -> T>(
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

        let function: Function = exports.get(export_names[idx as usize].as_str()).unwrap();

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
        extism_pdk::info!("ARG START");
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
        extism_pdk::info!("ARG: {}", arg);
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
        #[allow(clippy::blocks_in_conditions)]
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
    unwrap_value!(
        -1,
        invoke(idx, |_ctx, r| r.as_number().unwrap_or_default() as i32)
    )
}

#[no_mangle]
pub extern "C" fn __invoke_i64(idx: i32) -> i64 {
    unwrap_value!(
        -1,
        invoke(idx, |_ctx, r| {
            let Some(number) = r.as_big_int() else {
                return 0;
            };
            number.clone().to_i64().unwrap_or_default()
        })
    )
}

#[no_mangle]
pub extern "C" fn __invoke_f64(idx: i32) -> f64 {
    unwrap_value!(
        -1.0,
        invoke(idx, |_ctx, r| r.as_float().unwrap_or_default())
    )
}

#[no_mangle]
pub extern "C" fn __invoke_f32(idx: i32) -> f32 {
    unwrap_value!(
        -1.0,
        invoke(idx, |_ctx, r| r.as_number().unwrap_or_default() as f32)
    )
}

#[no_mangle]
pub extern "C" fn __invoke(idx: i32) {
    unwrap_value!((), invoke(idx, |_ctx, _r| ()))
}

fn export_names(exports: Object) -> anyhow::Result<Vec<String>> {
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
