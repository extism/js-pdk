use extism_pdk::bindings::extism_output_set;
use quickjs_wasm_rs::Context;
use extism_pdk::*;
use std::io::{Read};
use std::io;
use once_cell::sync::OnceCell;

mod globals;

static mut CONTEXT: OnceCell<Context> = OnceCell::new();
static mut CODE: OnceCell<String> = OnceCell::new();

#[export_name = "wizer.initialize"]
pub extern "C" fn init() {
    let context = Context::default();
    globals::inject_globals(&context, io::stderr(), io::stderr()).unwrap();

    let mut contents = String::new();
    io::stdin().read_to_string(&mut contents).unwrap();

    // we could preload the whole script here
    //let _ = context.eval_global("script.js", &contents).unwrap();

    unsafe {
        CONTEXT.set(context).unwrap();
        CODE.set(contents).unwrap();
    }
}

// #[plugin_fn]
// pub fn __invoke(func_name: String) -> FnResult<String> {
//     Ok(invoke(func_name)?)
// }

#[no_mangle]
pub unsafe extern "C" fn __invoke(func_idx: i32) -> i32 {
    let code = unsafe { CODE.take().unwrap() };
    let context = unsafe { CONTEXT.take().unwrap() };

    let log_stream =  io::stderr();
    let error_stream = io::stderr();
    globals::inject_globals(&context, log_stream, error_stream).expect("Failed to initialize globals");

    let _ = context.eval_global("script.js", &code).unwrap();

    let export_funcs = export_names(&context).expect("Could not parse exports");
    let func_name = export_funcs.get(func_idx as usize).unwrap();
    let result = context.eval_global("script.js", format!("{}();", func_name).as_str()).expect("Could not invoke");
    extism_pdk::output(result.as_str().expect("Could not convert result to str")).expect("Could not set the output");
    0
}


// #[no_mangle]
// pub fn greet() -> i32 {
//     unsafe {__invoke(1)}
// }

// #[plugin_fn]
// pub fn exports(_: ()) -> FnResult<String> {
//     Ok(export_names()?)
// }

// #[plugin_fn]
// pub fn count_vowels(_: ()) -> FnResult<String> {
//     Ok(invoke("count_vowels".into())?)
// }

fn export_names(context: &Context) -> anyhow::Result<Vec<String>> {
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
