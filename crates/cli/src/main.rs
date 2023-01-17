mod opt;
mod options;

use crate::options::Options;
use anyhow::{bail, Result};
use quick_js::Context;
use std::env;
use std::io::{Read, Write};
use std::path::Path;
use std::process::Stdio;
use std::{fs, process::Command};
use structopt::StructOpt;

fn main() -> Result<()> {
    let opts = Options::from_args();
    let wizen = env::var("EXTISM_WIZEN");

    if wizen.eq(&Ok("1".into())) {
        let wasm: &[u8] = include_bytes!(concat!(env!("OUT_DIR"), "/engine.wasm"));
        opt::Optimizer::new(wasm)
            .optimize(true)
            .write_optimized_wasm(opts.output)?;

        env::remove_var("EXTISM_WIZEN");

        return Ok(());
    }

    let mut input_file = fs::File::open(&opts.input)?;
    let mut contents: Vec<u8> = vec![];
    input_file.read_to_end(&mut contents)?;

    let self_cmd = env::args().next().unwrap();

    {
        env::set_var("EXTISM_WIZEN", "1");
        let mut command = Command::new(self_cmd)
            .arg(&opts.input)
            .arg("-o")
            .arg(&opts.output)
            .stdin(Stdio::piped())
            .spawn()?;
        command.stdin.take().unwrap().write_all(&contents)?;
        let status = command.wait()?;
        if !status.success() {
            bail!("Couldn't create wasm from input");
        }
    }

    add_extism_shim_exports(&opts.output, contents)?;

    Ok(())
}

fn add_extism_shim_exports<P: AsRef<Path>>(file: P, contents: Vec<u8>) -> Result<()> {
    use parity_wasm::elements::*;

    let code = String::from_utf8(contents)?;

    let context = Context::new().unwrap();
    let _ = context.eval("module = {exports: {}}").unwrap();
    let _ = context.eval(&code).unwrap();
    let global_functions = context
        .eval_as::<Vec<String>>("Object.keys(module.exports)")
        .unwrap();

    let mut exported_functions: Vec<String> = global_functions
        .into_iter()
        .filter(|name| name != "module")
        .collect();
    exported_functions.sort();

    let mut module = parity_wasm::deserialize_file(&file)?;

    let invoke_func_idx = if let Some(Internal::Function(idx)) = module
        .export_section()
        .unwrap()
        .entries()
        .iter()
        .find_map(|e| {
            if e.field() == "__invoke" {
                Some(e.internal())
            } else {
                None
            }
        }) {
        idx
    } else {
        bail!("Could not find __invoke function")
    };

    let wrapper_type_idx = module
        .type_section()
        .unwrap()
        .types()
        .iter()
        .enumerate()
        .find_map(|(idx, t)| {
            let Type::Function(ft) = t;
            // we are looking for the function (type () (result i32))
            // it takes no params and returns an i32. this is the extism call interface
            if ft.params() == vec![] && ft.results() == vec![ValueType::I32] {
                Some(idx)
            } else {
                None
            }
        });

    // TODO create the type if it doesn't exist
    let wrapper_type_idx = wrapper_type_idx.unwrap();

    let mut function_bodies = vec![];

    for (func_id, _export_name) in exported_functions.iter().enumerate() {
        function_bodies.push(FuncBody::new(
            vec![],
            Instructions::new(vec![
                Instruction::I32Const(func_id as i32),
                Instruction::Call(*invoke_func_idx),
                Instruction::End,
            ]),
        ));
    }

    for (idx, f) in function_bodies.into_iter().enumerate() {
        // put the code body in the code section
        let bodies = module.code_section_mut().unwrap().bodies_mut();
        bodies.push(f);

        // put the function type in the function section table
        let func = Func::new(wrapper_type_idx as u32);
        module
            .function_section_mut()
            .unwrap()
            .entries_mut()
            .push(func);

        //get the index of the function we just made
        let max_func_index = module.functions_space() - 1;

        // put the function in the exports table
        let export_section = module.export_section_mut().unwrap();
        let entry = ExportEntry::new(
            exported_functions.get(idx).unwrap().to_string(),
            Internal::Function(max_func_index as u32),
        );
        export_section.entries_mut().push(entry);
    }

    parity_wasm::serialize_to_file(&file, module)?;

    Ok(())
}
