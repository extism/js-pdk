use extism_pdk::*;
use once_cell::sync::OnceCell;
use quickjs_wasm_rs::JSContextRef;
use std::io;
use std::io::Read;

mod globals;

static mut CONTEXT: OnceCell<JSContextRef> = OnceCell::new();

#[link(wasm_import_module = "codemod")]
extern "C" {
    fn __invokeHostFunc(func_idx: u64, a: u64) -> u64;
}

#[export_name = "wizer.initialize"]
pub extern "C" fn init() {
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

#[no_mangle]
pub unsafe extern "C" fn __invoke(func_idx: i32) -> i32 {
    let context = unsafe { CONTEXT.get().unwrap() };

    let export_funcs = export_names(&context).expect("Could not parse exports");
    let func_name = export_funcs
        .get(func_idx as usize)
        .expect(format!("Could not find export func at index {func_idx}").as_str());
    let result = context.eval_global("script.js", format!("{}();", func_name).as_str());

    unwrap!(result);
    0
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
