use extism_pdk::bindings::extism_error_set;
use extism_pdk::Memory;
use once_cell::sync::OnceCell;
use quickjs_wasm_rs::Context;
use std::io;
use std::io::Read;

mod globals;

static mut CONTEXT: OnceCell<Context> = OnceCell::new();
static mut CODE: OnceCell<String> = OnceCell::new();

#[export_name = "wizer.initialize"]
pub extern "C" fn init() {
    let context = Context::default();
    globals::inject_globals(&context).expect("Failed to initialize globals");

    let mut contents = String::new();
    io::stdin().read_to_string(&mut contents).unwrap();

    // we could preload the whole script here
    //let _ = context.eval_global("script.js", &contents).unwrap();

    unsafe {
        CONTEXT.set(context).unwrap();
        CODE.set(contents).unwrap();
    }
}

#[no_mangle]
pub unsafe extern "C" fn __invoke(func_idx: i32) -> i32 {
    let code = unsafe { CODE.take().unwrap() };
    let context = unsafe { CONTEXT.take().unwrap() };

    globals::inject_globals(&context).expect("Failed to initialize globals");

    let _ = context
        .eval_global("script.js", &code)
        .expect("Could not eval main script");

    let export_funcs = export_names(&context).expect("Could not parse exports");
    let func_name = export_funcs
        .get(func_idx as usize)
        .expect(format!("Could not find export func at index {func_idx}").as_str());
    let result = context
        .eval_global("script.js", format!("{}();", func_name).as_str());

    match result {
        Ok(r) => r.as_i32_unchecked(),
        Err(e) => {
            let err = format!("{:?}", e);
            let mut mem = Memory::new(err.len());
            mem.store(err.as_bytes());
            unsafe {
                extism_error_set(mem.offset);
            }
            -1
        }
    }
}

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
