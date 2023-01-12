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


#[plugin_fn]
pub fn run_js_code(_: ()) -> FnResult<String> {
    Ok(exports("function add() { return 'Hello' }; exports = {add};")?)
}

#[plugin_fn]
pub fn __invoke(func_name: String) -> FnResult<String> {
    Ok(invoke(func_name)?)
}

fn invoke(func_name: String) -> anyhow::Result<String> {
    let code = unsafe { CODE.take().unwrap() };
    let context = unsafe { CONTEXT.take().unwrap() };

    let _ = context.eval_global("script.js", &code).unwrap();
    //let global = context.global_object().unwrap();
    let result = context.eval_global("script.js", format!("{}();", func_name).as_str())?;
    Ok(format!("{:#?}", result.as_str()?))
}

#[plugin_fn]
pub fn greet(_: ()) -> FnResult<String> {
    Ok(invoke("greet".into())?)
}

#[plugin_fn]
pub fn count_vowels(_: ()) -> FnResult<String> {
    Ok(invoke("count_vowels".into())?)
}

pub fn exports(contents: &str) -> anyhow::Result<String> {
    let context = Context::default();
    let log_stream =  io::stderr();
    let error_stream = io::stderr();

    globals::inject_globals(&context, log_stream, error_stream)?;

    //let contents = "function add(a, b) { return 'Hello From Javascript' }; exports = { add } ";
    let _ = context.eval_global("script.js", &contents).unwrap();
    let global = context.global_object().unwrap();
    //let module = global.get_property("module")?;
    let exports = global.get_property("exports")?;
    let mut properties = exports.properties()?;
    let mut key = properties.next_key()?;
    let mut keys: Vec<String> = vec![];
    while key.is_some() {
       keys.push(key.unwrap().as_str()?.to_string());
        key = properties.next_key()?;
    }
    Ok(keys.join(","))
}
