extern crate swc_common;
extern crate swc_ecma_parser;
use anyhow::Result;
use std::fs::File;
use std::io::prelude::*;
use std::path::PathBuf;

use wasm_encoder::{
    CodeSection, ConstExpr, ElementSection, Elements, EntityType, ExportKind, ExportSection,
    Function, FunctionSection, HeapType, Instruction, TableSection, TableType, TypeSection,
    ValType,
};
use wasm_encoder::{ImportSection, Module as WasmModule};

use crate::interface_parser::{Interface, PluginInterface};

/// Generates the wasm shim for the exports
fn generate_wasm_shims(
    exports: Interface,
    export_path: &PathBuf,
    imports: Option<Interface>,
    import_path: &PathBuf,
) -> Result<()> {
    let mut export_mod = WasmModule::new();

    // Note: the order in which you set the sections
    // with `export_mod.section()` is important

    // Encode the type section.
    let mut types = TypeSection::new();
    // __invoke's type
    let params = vec![ValType::I32];
    let results = vec![ValType::I32];
    types.function(params, results);
    // Extism Export type
    let params = vec![];
    let results = vec![ValType::I32];
    types.function(params, results);
    export_mod.section(&types);

    //Encode the import section
    let mut import_sec = ImportSection::new();
    import_sec.import("coremod", "__invoke", EntityType::Function(0));
    export_mod.section(&import_sec);

    // Encode the function section.
    let mut functions = FunctionSection::new();

    // we will have 1 thunk function per export
    let type_index = 1; // these are exports () -> i32
    for _ in exports.functions.iter() {
        functions.function(type_index);
    }
    export_mod.section(&functions);

    let mut func_index = 1;

    // Encode the export section.
    let mut export_sec = ExportSection::new();
    // we need to sort them alphabetically because that is
    // how the runtime maps indexes
    let mut export_functions = exports.functions.clone();
    export_functions.sort_by(|a, b| a.name.cmp(&b.name));
    for i in export_functions.iter() {
        export_sec.export(i.name.as_str(), ExportKind::Func, func_index);
        func_index += 1;
    }
    export_mod.section(&export_sec);

    // Encode the code section.
    let mut codes = CodeSection::new();
    let mut export_idx: i32 = 0;

    // create a single thunk per export
    for _ in exports.functions.iter() {
        let locals = vec![];
        let mut f = Function::new(locals);
        // we will essentially call the eval function (__invoke)
        f.instruction(&Instruction::I32Const(export_idx));
        f.instruction(&Instruction::Call(0));
        f.instruction(&Instruction::End);
        codes.function(&f);
        export_idx += 1;
    }
    export_mod.section(&codes);

    // Extract the encoded Wasm bytes for this module.
    let wasm_bytes = export_mod.finish();
    let mut file = File::create(export_path)?;
    file.write_all(wasm_bytes.as_ref())?;

    if imports.is_none() {
        return Ok(());
    }

    let imports = imports.unwrap();

    let mut import_mod = WasmModule::new();

    // Encode the type section.
    let mut types = TypeSection::new();

    // for __invokeHostFunc
    let params = vec![ValType::I64, ValType::I64];
    let results = vec![ValType::I64];
    types.function(params, results);

    // for all other host funcs (TODO fix)
    let params = vec![ValType::I64];
    let results = vec![ValType::I64];
    types.function(params, results);
    import_mod.section(&types);

    // Encode the import section
    let mut import_sec = ImportSection::new();

    for i in imports.functions.iter() {
        import_sec.import(
            "extism:host/user",
            i.name.as_str(),
            wasm_encoder::EntityType::Function(1),
        );
    }
    import_mod.section(&import_sec);

    // Encode the function section.
    let mut functions = FunctionSection::new();
    functions.function(0);
    import_mod.section(&functions);

    // encode tables pointing to imports
    let mut tables = TableSection::new();
    let table_type = TableType {
        element_type: wasm_encoder::RefType {
            nullable: true,
            heap_type: HeapType::Func,
        },
        minimum: 2,
        maximum: None,
    };
    tables.table(table_type);
    import_mod.section(&tables);

    // Encode the export section.
    let mut export_sec = ExportSection::new();
    export_sec.export(
        "__invokeHostFunc",
        ExportKind::Func,
        imports.functions.len() as u32, // will be the last function
    );
    import_mod.section(&export_sec);

    // Encode the element section.
    let mut elements = ElementSection::new();
    let func_elems = Elements::Functions(&[0, 1]);
    let offset = ConstExpr::i32_const(0);
    elements.active(None, &offset, func_elems);
    import_mod.section(&elements);

    // Encode the code section.
    let mut codes = CodeSection::new();
    let locals = vec![];
    let mut f = Function::new(locals);
    // we will essentially call the eval function
    // in the core module here, similar to https://github.com/extism/js-pdk/blob/eaf17366624d48219cbd97a51e85569cffd12086/crates/cli/src/main.rs#L118
    f.instruction(&Instruction::LocalGet(0));
    f.instruction(&Instruction::LocalGet(1));
    //f.instruction(&Instruction::I32Const(0));
    f.instruction(&Instruction::CallIndirect { ty: 0, table: 0 });
    f.instruction(&Instruction::End);
    codes.function(&f);
    import_mod.section(&codes);

    let wasm_bytes = import_mod.finish();
    let mut file = File::create(import_path)?;
    file.write_all(wasm_bytes.as_ref())?;

    Ok(())
}

pub fn create_shims(
    plugin_interface: &PluginInterface,
    export_path: &PathBuf,
    import_path: &PathBuf,
) -> Result<()> {
    generate_wasm_shims(
        plugin_interface.exports.clone(),
        export_path,
        plugin_interface.imports.clone(),
        import_path,
    )?;
    Ok(())
}
