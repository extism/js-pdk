use rquickjs::{
    function::Args, object::ObjectKeysIter, Context, Ctx, Function, Object, Runtime, Undefined,
    Value,
};
use std::io;
use std::io::Read;

mod globals;

struct Cx(Context);

unsafe impl Send for Cx {}
unsafe impl Sync for Cx {}

static CONTEXT: std::sync::OnceLock<Cx> = std::sync::OnceLock::new();
static CALL_ARGS: std::sync::Mutex<Vec<Vec<ArgType>>> = std::sync::Mutex::new(vec![]);

fn check_exception(this: &Ctx, err: rquickjs::Error) -> anyhow::Error {
    let s = match err {
        rquickjs::Error::Exception => {
            let err = this.catch().into_exception().unwrap();
            let msg = err.message().unwrap_or_default();
            format!("Exception: {}\n{}", msg, err.stack().unwrap_or_default())
        }
        err => err.to_string(),
    };

    let mem = extism_pdk::Memory::from_bytes(&s).unwrap();
    unsafe {
        extism_pdk::extism::error_set(mem.offset());
    }
    anyhow::Error::msg(s)
}

#[export_name = "wizer.initialize"]
extern "C" fn init() {
    let runtime = Runtime::new().expect("Couldn't make a runtime");
    let context = Context::full(&runtime).expect("Couldnt make a context");
    globals::inject_globals(&context).expect("Failed to initialize globals");

    let mut code = String::new();
    io::stdin().read_to_string(&mut code).unwrap();

    context
        .with(|this| -> Result<rquickjs::Undefined, anyhow::Error> {
            match this.eval(code) {
                Ok(()) => (),
                Err(err) => return Err(check_exception(&this, err)),
            }
            Ok(Undefined)
        })
        .unwrap();
    let _ = CONTEXT.set(Cx(context));
}

fn js_context() -> Context {
    if CONTEXT.get().is_none() {
        init()
    }
    let context = CONTEXT.get().unwrap();
    context.0.clone()
}

fn invoke<'a, T, F: for<'b> Fn(Ctx<'b>, Value<'b>) -> T>(
    idx: i32,
    conv: F,
) -> Result<T, anyhow::Error> {
    let call_args = CALL_ARGS.lock().unwrap().pop();
    let context = js_context();
    context.with(|ctx| {
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
            Err(err) => Err(check_exception(&ctx, err)),
        }
    })
}

#[no_mangle]
pub extern "C" fn __arg_start() {
    CALL_ARGS.lock().unwrap().push(vec![]);
}

#[no_mangle]
pub extern "C" fn __arg_i32(arg: i32) {
    CALL_ARGS
        .lock()
        .unwrap()
        .last_mut()
        .unwrap()
        .push(ArgType::I32(arg));
}

#[no_mangle]
pub extern "C" fn __arg_i64(arg: i64) {
    CALL_ARGS
        .lock()
        .unwrap()
        .last_mut()
        .unwrap()
        .push(ArgType::I64(arg));
}

#[no_mangle]
pub extern "C" fn __arg_f32(arg: f32) {
    CALL_ARGS
        .lock()
        .unwrap()
        .last_mut()
        .unwrap()
        .push(ArgType::F32(arg));
}

#[no_mangle]
pub extern "C" fn __arg_f64(arg: f64) {
    CALL_ARGS
        .lock()
        .unwrap()
        .last_mut()
        .unwrap()
        .push(ArgType::F64(arg));
}

#[no_mangle]
pub extern "C" fn __invoke_i32(idx: i32) -> i32 {
    invoke(idx, |_ctx, r| r.as_number().unwrap_or_default() as i32).unwrap_or(-1)
}

#[no_mangle]
pub extern "C" fn __invoke_i64(idx: i32) -> i64 {
    invoke(idx, |_ctx, r| {
        if let Some(number) = r.as_big_int() {
            return number.clone().to_i64().unwrap_or_default();
        } else if let Some(number) = r.as_number() {
            return number as i64;
        }
        0
    })
    .unwrap_or(-1)
}

#[no_mangle]
pub extern "C" fn __invoke_f64(idx: i32) -> f64 {
    invoke(idx, |_ctx, r| r.as_float().unwrap_or_default()).unwrap_or(-1.0)
}

#[no_mangle]
pub extern "C" fn __invoke_f32(idx: i32) -> f32 {
    invoke(idx, |_ctx, r| r.as_number().unwrap_or_default() as f32).unwrap_or(-1.0)
}

#[no_mangle]
pub extern "C" fn __invoke(idx: i32) {
    invoke(idx, |_ctx, _r| ()).unwrap()
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
