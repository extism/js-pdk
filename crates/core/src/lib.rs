use once_cell::sync::OnceCell;
use quickjs_wasm_rs::{JSContextRef, JSValue, JSValueRef};
use std::io;
use std::io::Read;

mod globals;

static mut CONTEXT: OnceCell<JSContextRef> = OnceCell::new();
static mut CALL_ARGS: Vec<Vec<JSValue>> = vec![];

#[export_name = "wizer.initialize"]
extern "C" fn init() {
    let context = JSContextRef::default();
    globals::inject_globals(&context).expect("Failed to initialize globals");

    let mut code = String::new();
    io::stdin().read_to_string(&mut code).unwrap();

    let _ = context
        .eval_global("script.js", &code)
        .expect("Could not eval main script");

    unsafe {
        CONTEXT.set(context).unwrap();
    }
}

fn js_context<'a>() -> &'a JSContextRef {
    unsafe {
        if CONTEXT.get().is_none() {
            init()
        }

        let context = CONTEXT.get_unchecked();
        context
    }
}

fn convert_js_value<'a>(context: &'a JSContextRef, v: &JSValue) -> JSValueRef<'a> {
    match v {
        JSValue::Undefined => context.undefined_value().unwrap(),
        JSValue::Null => context.null_value().unwrap(),
        JSValue::Bool(b) => context.value_from_bool(*b).unwrap(),
        JSValue::Int(i) => context.value_from_i32(*i).unwrap(),
        JSValue::Float(f) => context.value_from_f64(*f).unwrap(),
        JSValue::String(s) => context.value_from_str(s.as_str()).unwrap(),
        JSValue::Array(a) => {
            let arr = context.array_value().unwrap();
            for x in a.iter() {
                arr.append_property(convert_js_value(context, x)).unwrap();
            }
            arr
        }
        JSValue::ArrayBuffer(buf) => context.array_buffer_value(buf.as_slice()).unwrap(),
        JSValue::Object(x) => {
            let obj = context.object_value().unwrap();
            for (k, v) in x.iter() {
                obj.set_property(k.as_str(), convert_js_value(context, v))
                    .unwrap();
            }
            obj
        }
    }
}

fn invoke<'a, T, F: Fn(&'a JSContextRef, JSValueRef<'a>) -> T>(idx: i32, conv: F) -> T {
    let call_args = unsafe { CALL_ARGS.pop() };
    let context = js_context();
    let args: Vec<_> = call_args
        .unwrap()
        .iter()
        .map(|x| convert_js_value(context, x))
        .collect();
    let globals = context.global_object().unwrap();
    let names = export_names(&context).unwrap();
    let f = globals.get_property(names[idx as usize].as_str()).unwrap();
    let r = f.call(&context.undefined_value().unwrap(), &args).unwrap();
    conv(&context, r)
}

// #[no_mangle]
// pub fn __invokeHostFunc(idx: i32, i: u64) -> u64 {
//     let call_args = unsafe { CALL_ARGS.pop() };
//     let context = js_context();
//     let args: Vec<_> = call_args
//         .unwrap()
//         .iter()
//         .map(|x| convert_js_value(context, x))
//         .collect();
//     let globals = context.global_object().unwrap();
//     let names = export_names(&context).unwrap();
//     let f = globals.get_property(names[idx as usize].as_str()).unwrap();

//     let r = f.call(&context.undefined_value().unwrap(), &args).unwrap();
// }

#[no_mangle]
pub extern "C" fn __arg_start() {
    unsafe {
        CALL_ARGS.push(vec![]);
    }
}

#[no_mangle]
pub extern "C" fn __arg_i32(arg: i32) {
    unsafe {
        CALL_ARGS.last_mut().unwrap().push(JSValue::Int(arg));
    }
}

#[no_mangle]
pub extern "C" fn __arg_i64(arg: i64) {
    unsafe {
        CALL_ARGS
            .last_mut()
            .unwrap()
            .push(JSValue::Float(arg as f64));
    }
}

#[no_mangle]
pub extern "C" fn __arg_f32(arg: f32) {
    unsafe {
        CALL_ARGS
            .last_mut()
            .unwrap()
            .push(JSValue::Float(arg as f64));
    }
}

#[no_mangle]
pub extern "C" fn __arg_f64(arg: f64) {
    unsafe {
        CALL_ARGS.last_mut().unwrap().push(JSValue::Float(arg));
    }
}

#[no_mangle]
pub extern "C" fn __invoke_i32(idx: i32) -> i32 {
    invoke(idx, |_ctx, r| r.as_i32_unchecked())
}

#[no_mangle]
pub extern "C" fn __invoke_i64(idx: i32) -> i64 {
    invoke(idx, |_ctx, r| r.as_f64_unchecked() as i64)
}

#[no_mangle]
pub extern "C" fn __invoke_f64(idx: i32) -> f64 {
    invoke(idx, |_ctx, r| r.as_f64_unchecked())
}

#[no_mangle]
pub extern "C" fn __invoke_f32(idx: i32) -> f32 {
    invoke(idx, |_ctx, r| r.as_f64_unchecked() as f32)
}

#[no_mangle]
pub extern "C" fn __invoke(idx: i32) {
    invoke(idx, |_ctx, _r| ())
}

fn export_names(context: &JSContextRef) -> anyhow::Result<Vec<String>> {
    let global = context.global_object().unwrap();
    let module = global.get_property("module")?;
    let exports = module.get_property("exports")?;
    let mut properties = exports.properties()?;
    let mut key = properties.next_key()?;
    let mut keys: Vec<String> = vec![];
    while key.is_some() {
        keys.push(key.unwrap().as_str()?.to_string());
        key = properties.next_key()?;
    }
    keys.sort();
    Ok(keys)
}
