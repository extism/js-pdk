use crate::ts_parser::Interface;
use anyhow::Result;
use std::path::Path;
use wagen::{BlockType, Instr, ValType};

#[derive(PartialEq)]
enum TypeCode {
    Void = 0,
    I32 = 1,
    I64 = 2,
    F32 = 3,
    F64 = 4,
}

pub fn generate_wasm_shims(
    path: impl AsRef<Path>,
    exports: &Interface,
    imports: &[Interface],
) -> Result<()> {
    let mut module = wagen::Module::new();

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

    let mut import_elements = Vec::new();
    let mut import_items = vec![];
    for import in imports.iter() {
        for f in import.functions.iter() {
            let params: Vec<_> = f.params.iter().map(|x| x.ptype).collect();
            let results: Vec<_> = f.results.iter().map(|x| x.ptype).collect();
            let index = module.import(&import.name, &f.name, None, params.clone(), results.clone());
            import_items.push((f.name.clone(), index, params, results));
        }
    }
    import_items.sort_by_key(|x| x.0.to_string());

    for (_name, index, _params, _results) in &import_items {
        import_elements.push(index.index());
    }

    let table_min = import_elements.len() as u32;

    let import_table = module.tables().push(wagen::TableType {
        element_type: wagen::RefType::FUNCREF,
        minimum: table_min,
        maximum: None,
    });

    let mut get_function_return_type_builder = wagen::Builder::default();

    for (func_idx, (_name, _index, _params, results)) in import_items.iter().enumerate() {
        let type_code = results.first().map_or(TypeCode::Void, |val_type| match val_type {
            ValType::I32 => TypeCode::I32,
            ValType::I64 => TypeCode::I64,
            ValType::F32 => TypeCode::F32,
            ValType::F64 => TypeCode::F64,
            _ => TypeCode::Void,
        });
        
        if type_code == TypeCode::Void {
            continue;
        }

        // Compare the input function index with the current index.
        get_function_return_type_builder.push(Instr::LocalGet(0)); // load requested function index
        get_function_return_type_builder.push(Instr::I32Const(func_idx as i32)); // load func_idx
        get_function_return_type_builder.push(Instr::I32Eq); // compare
        get_function_return_type_builder.push(Instr::If(BlockType::Empty)); // if true
        get_function_return_type_builder.push(Instr::I32Const(type_code as i32)); // load type code
        get_function_return_type_builder.push(Instr::Return); // early return if match
        get_function_return_type_builder.push(Instr::End);
    }

    get_function_return_type_builder.push(Instr::I32Const(0)); // Default to 0
    get_function_return_type_builder.push(Instr::Return);

    let get_function_return_type_func = module.func(
        "__get_function_return_type",
        vec![ValType::I32], // takes function index
        vec![ValType::I32], // returns type code
        vec![],
    );
    get_function_return_type_func.export("__get_function_return_type");
    get_function_return_type_func.body = get_function_return_type_builder;

    let mut get_function_arg_type_builder = wagen::Builder::default();

    for (func_idx, (_name, _index, params, _results)) in import_items.iter().enumerate() {
        for arg_idx in 0..params.len() {
            let type_code = match params[arg_idx] {
                ValType::I32 => TypeCode::I32,
                ValType::I64 => TypeCode::I64,
                ValType::F32 => TypeCode::F32,
                ValType::F64 => TypeCode::F64,
                _ => panic!("Unsupported argument type for function {} at index {}", func_idx, arg_idx),
            };

            // Compare both function index and argument index
            get_function_arg_type_builder.push(Instr::LocalGet(0)); // function index
            get_function_arg_type_builder.push(Instr::I32Const(func_idx as i32));
            get_function_arg_type_builder.push(Instr::I32Eq);

            get_function_arg_type_builder.push(Instr::LocalGet(1)); // argument index
            get_function_arg_type_builder.push(Instr::I32Const(arg_idx as i32));
            get_function_arg_type_builder.push(Instr::I32Eq);

            get_function_arg_type_builder.push(Instr::I32And); // Both must match

            // If both match, return the type code
            get_function_arg_type_builder.push(Instr::If(BlockType::Empty));
            get_function_arg_type_builder.push(Instr::I32Const(type_code as i32));
            get_function_arg_type_builder.push(Instr::Return);
            get_function_arg_type_builder.push(Instr::End);
        }
    }

    // Default return if no match
    get_function_arg_type_builder.push(Instr::I32Const(0));
    get_function_arg_type_builder.push(Instr::Return);

    let get_function_arg_type_func = module.func(
        "__get_function_arg_type",
        vec![ValType::I32, ValType::I32], // takes (function_index, arg_index)
        vec![ValType::I32],               // returns type code
        vec![],
    );
    get_function_arg_type_func.export("__get_function_arg_type");
    get_function_arg_type_func.body = get_function_arg_type_builder;

    // Create converters for each host function to reinterpret the I64 bit pattern as the expected type
    let mut converter_indices = Vec::new();
    for (_, (name, _index, params, results)) in import_items.iter().enumerate() {
        let import_type = module
            .types()
            .push(|t| t.function(params.clone(), results.clone()));

        let mut builder = wagen::Builder::default();

        // Convert input parameters
        for (i, param) in params.iter().enumerate() {
            builder.push(Instr::LocalGet((i + 1) as u32)); // skip function index param

            match param {
                ValType::I32 => {
                    builder.push(Instr::I32WrapI64);
                }
                ValType::I64 => {
                    // No conversion needed - already i64
                }
                ValType::F32 => {
                    // Input is already the bit pattern from globals.rs convert_to_u64_bits
                    // First truncate to i32 then reinterpret as f32
                    builder.push(Instr::I32WrapI64);
                    builder.push(Instr::F32ReinterpretI32);
                }
                ValType::F64 => {
                    // Input is already the bit pattern from JS DataView
                    // Just reinterpret the i64 as f64
                    builder.push(Instr::F64ReinterpretI64);
                }
                r => {
                    anyhow::bail!("Unsupported param type: {:?}", r);
                }
            }
        }

        // Call the imported function
        builder.push(Instr::LocalGet(0));
        builder.push(Instr::CallIndirect {
            ty: import_type,
            table: import_table,
        });

        // Convert result back to i64 bits for JS
        if let Some(result) = results.first() {
            match result {
                ValType::I32 => {
                    builder.push(Instr::I64ExtendI32U);
                }
                ValType::I64 => {
                    // Already i64, no conversion needed
                }
                ValType::F32 => {
                    // Convert f32 to its bit pattern
                    builder.push(Instr::I32ReinterpretF32);
                    builder.push(Instr::I64ExtendI32U);
                }
                ValType::F64 => {
                    // Convert f64 to its bit pattern
                    builder.push(Instr::I64ReinterpretF64);
                }
                r => {
                    anyhow::bail!("Unsupported result type: {:?}", r);
                }
            }
        } else {
            // No return value, push 0
            builder.push(Instr::I64Const(0));
        }

        // Create the converter function
        let mut shim_params = vec![ValType::I32]; // Function index
        shim_params.extend(std::iter::repeat(ValType::I64).take(params.len()));

        let conv_func = module.func(
            &format!("__conv_{}", name),
            shim_params,
            vec![ValType::I64],
            vec![],
        );
        conv_func.export(&format!("__conv_{}", name));
        conv_func.body = builder;

        converter_indices.push(conv_func.index);
    }

    let router = module.func(
        "__invokeHostFunc",
        vec![
            ValType::I32, // func_idx
            ValType::I64, // args[0]
            ValType::I64, // args[1]
            ValType::I64, // args[2]
            ValType::I64, // args[3]
            ValType::I64, // args[4]
        ],
        vec![ValType::I64],
        vec![],
    );

    // Similar builder logic as before but simplified to one function
    let mut router_builder = wagen::Builder::default();

    for (func_idx, (_name, _index, params, _results)) in import_items.iter().enumerate() {
        router_builder.push(Instr::LocalGet(0)); // func index
        router_builder.push(Instr::I32Const(func_idx as i32));
        router_builder.push(Instr::I32Eq);
        router_builder.push(Instr::If(BlockType::Empty));

        // First push func_idx for converter
        router_builder.push(Instr::LocalGet(0));

        // Then push remaining args from router's inputs
        for (i, _) in params.iter().enumerate() {
            router_builder.push(Instr::LocalGet((i + 1) as u32));
        }

        router_builder.push(Instr::Call(converter_indices[func_idx]));
        router_builder.push(Instr::Return);
        router_builder.push(Instr::End);
    }

    router_builder.push(Instr::I64Const(0));
    router_builder.push(Instr::Return);

    router.export("__invokeHostFunc");
    router.body = router_builder;

    // Set up the table
    module.active_element(
        Some(import_table),
        wagen::Elements::Functions(&import_elements),
    );

    // Generate exports
    for (idx, export) in exports.functions.iter().enumerate() {
        let params: Vec<_> = export.params.iter().map(|x| x.ptype).collect();
        let results: Vec<_> = export.results.iter().map(|x| x.ptype).collect();
        if results.len() > 1 {
            anyhow::bail!(
                "Multiple return arguments are not currently supported but used in exported function {}",
                export.name
            );
        }

        let mut builder = wagen::Builder::default();
        builder.push(Instr::Call(__arg_start.index()));

        for (parami, param) in params.iter().enumerate() {
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

        let f = module.func(&export.name, params, results, vec![]);
        f.export(&export.name);
        f.body = builder;
    }

    // Validation with debug output
    if let Err(error) = module.clone().validate_save(path.as_ref()) {
        eprintln!("Validation failed: {:?}", error);
        module.save("/tmp/wizer/incomplete_shim.wasm")?;
        return Err(error);
    }

    Ok(())
}
