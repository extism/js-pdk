use anyhow::Result;
use std::{collections::HashMap, path::Path};

use crate::ts_parser::Interface;
use wagen::{Instr, ValType};

/// Generates the wasm shim for the exports
pub fn generate_wasm_shims(
    path: impl AsRef<Path>,
    exports: &Interface,
    imports: &[Interface],
) -> Result<()> {
    let mut module = wagen::Module::new();

    let mut functions = HashMap::new();
    let __arg_start = module.import("core", "__arg_start", None, [], []);
    let __arg_i32 = module.import("core", "__arg_i32", None, [ValType::I32], []);
    let __arg_i64 = module.import("core", "__arg_i64", None, [ValType::I64], []);
    let __arg_f32 = module.import("core", "__arg_f32", None, [ValType::F32], []);
    let __arg_f64 = module.import("core", "__arg_f64", None, [ValType::F64], []);
    let __invoke_i32 = module.import("core", "__invoke_i32", None, [ValType::I32], [ValType::I32]);
    let __invoke_i64 = module.import("core", "__invoke_i64", None, [ValType::I32], [ValType::I64]);
    let __invoke_f32 = module.import("core", "__invoke_f32", None, [ValType::I32], [ValType::F32]);
    let __invoke_f64 = module.import("core", "__invoke_f64", None, [ValType::I32], [ValType::F64]);
    let __invoke = module.import("core", "__invoke", None, [ValType::I32], []);

    for import in imports.iter() {
        for f in import.functions.iter() {
            let params: Vec<_> = f.params.iter().map(|x| x.ptype).collect();
            let results: Vec<_> = f.results.iter().map(|x| x.ptype).collect();
            let index = module.import(&import.name, &f.name, None, params, results);
            functions.insert(f.name.as_str(), index.index());
        }
    }

    for (idx, export) in exports.functions.iter().enumerate() {
        let params: Vec<_> = export.params.iter().map(|x| x.ptype).collect();
        let results: Vec<_> = export.results.iter().map(|x| x.ptype).collect();
        if results.len() > 1 {
            anyhow::bail!(
                "Multiple return arguments are not currently supported but used in exported function {}",
                export.name
            );
        }
        let func = module
            .func(&export.name, params.clone(), results.clone(), [])
            .export(&export.name);
        let builder = func.builder();
        builder.push(Instr::Call(__arg_start.index()));
        for (parami, param) in params.into_iter().enumerate() {
            builder.push(Instr::LocalGet(parami as u32));

            match param {
                ValType::I32 => {
                    builder.push(Instr::Call(__arg_i32.index()));
                }
                ValType::I64 => {
                    builder.push(Instr::Call(__arg_i64.index()));
                }
                ValType::F32 => {
                    builder.push(Instr::Call(__arg_f32.index()));
                }
                ValType::F64 => {
                    builder.push(Instr::Call(__arg_f64.index()));
                }
                r => {
                    anyhow::bail!("Unsupported param type: {:?}", r);
                }
            }
        }

        builder.push(Instr::I32Const(idx as i32));
        match results.first() {
            None => {
                builder.push(Instr::Call(__invoke.index()));
            }
            Some(ValType::I32) => {
                builder.push(Instr::Call(__invoke_i32.index()));
            }
            Some(ValType::I64) => {
                builder.push(Instr::Call(__invoke_i64.index()));
            }
            Some(ValType::F32) => {
                builder.push(Instr::Call(__invoke_f32.index()));
            }
            Some(ValType::F64) => {
                builder.push(Instr::Call(__invoke_f64.index()));
            }
            Some(r) => {
                anyhow::bail!("Unsupported result type: {:?}", r);
            }
        }
    }

    module.validate_save(path.as_ref())?;
    Ok(())
}
