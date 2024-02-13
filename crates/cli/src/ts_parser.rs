extern crate swc_common;
extern crate swc_ecma_parser;
use anyhow::{bail, Context, Result};
use std::path::PathBuf;
use wagen::ValType;

use swc_common::sync::Lrc;
use swc_common::SourceMap;
use swc_ecma_ast::{
    Decl, Module, ModuleDecl, Stmt, TsInterfaceDecl, TsKeywordTypeKind, TsModuleDecl, TsType,
};
use swc_ecma_ast::{ModuleItem, TsTypeElement};
use swc_ecma_parser::{lexer::Lexer, Parser, StringInput, Syntax};

#[derive(Debug, Clone)]
pub struct Param {
    pub name: String,
    pub ptype: ValType,
}

impl Param {
    pub fn new(name: &str, ptype: ValType) -> Param {
        Param {
            name: name.to_string(),
            ptype,
        }
    }
}

#[derive(Debug, Clone)]
pub struct Signature {
    pub name: String,
    pub params: Vec<Param>,
    pub results: Vec<Param>,
}

#[derive(Debug, Clone)]
pub struct Interface {
    pub name: String,
    pub functions: Vec<Signature>,
}

#[derive(Debug, Clone)]
pub struct PluginInterface {
    pub exports: Interface,
    pub imports: Vec<Interface>,
}

pub fn val_type(s: &str) -> ValType {
    match s.to_ascii_lowercase().as_str() {
        "i32" => ValType::I32,
        "i64" => ValType::I64,
        "f32" => ValType::F32,
        "f64" => ValType::F64,
        _ => ValType::I64, // Extism handle
    }
}

pub fn param_type(params: &mut Vec<Param>, vn: &str, t: &TsType) -> Result<()> {
    let typ = if let Some(t) = t.as_ts_type_ref() {
        t.type_name
            .as_ident()
            .context("Illegal param type")?
            .sym
            .as_str()
    } else {
        "i64"
    };
    params.push(Param::new(vn, val_type(typ)));
    Ok(())
}

pub fn result_type(results: &mut Vec<Param>, return_type: &TsType) -> Result<()> {
    let return_type = if let Some(return_type) = return_type.as_ts_type_ref() {
        Some(
            return_type
                .type_name
                .as_ident()
                .context("Illegal return type")?
                .sym
                .as_str(),
        )
    } else if let Some(t) = return_type.as_ts_keyword_type() {
        match t.kind {
            TsKeywordTypeKind::TsVoidKeyword
            | TsKeywordTypeKind::TsUndefinedKeyword
            | TsKeywordTypeKind::TsNullKeyword => None,
            _ => Some("i64"),
        }
    } else {
        Some("i64")
    };
    if let Some(r) = return_type {
        results.push(Param::new("result", val_type(r)));
    }
    Ok(())
}

/// Parses the non-main parts of the module which maps to the wasm imports
fn parse_user_interface(i: &Box<TsInterfaceDecl>) -> Result<Interface> {
    let mut signatures = Vec::new();
    let name = i.id.sym.as_str();
    for sig in &i.body.body {
        match sig {
            TsTypeElement::TsMethodSignature(t) => {
                let name = t.key.as_ident().unwrap().sym.to_string();
                let mut params = vec![];
                let mut results = vec![];

                for p in t.params.iter() {
                    let vn = p.as_ident().unwrap().id.sym.as_str();
                    let typ = p.as_ident().unwrap().type_ann.clone();
                    let t = typ.unwrap().type_ann;
                    param_type(&mut params, vn, &t)?;
                }
                if let Some(return_type) = &t.type_ann {
                    result_type(&mut results, &return_type.type_ann)?;
                }
                let signature = Signature {
                    name,
                    params,
                    results,
                };
                signatures.push(signature);
            }
            _ => {
                log::warn!("Warning: don't know what to do with sig {:#?}", sig);
            }
        }
    }

    Ok(Interface {
        name: name.into(),
        functions: signatures,
    })
}

/// Try to parse the imports
fn parse_imports(tsmod: &Box<TsModuleDecl>) -> Result<Option<Interface>> {
    for block in &tsmod.body {
        if let Some(block) = block.clone().ts_module_block() {
            for inter in block.body {
                if let ModuleItem::Stmt(Stmt::Decl(decl)) = inter {
                    let i = decl.as_ts_interface().unwrap();
                    let mut interface = parse_user_interface(i)?;
                    if tsmod.id.clone().str().is_some() {
                        interface.name = tsmod.id.clone().expect_str().value.as_str().to_string()
                            + "/"
                            + i.id.sym.as_str();
                    }
                    return Ok(Some(interface));
                } else {
                    log::warn!("Not a module decl");
                }
            }
        } else {
            log::warn!("Not a Module Block");
        }
    }
    Ok(None)
}

/// Parses the main module declaration (the extism exports)
fn parse_module_decl(tsmod: &Box<TsModuleDecl>) -> Result<Interface> {
    let mut signatures = Vec::new();

    for block in &tsmod.body {
        if let Some(block) = block.as_ts_module_block() {
            for decl in &block.body {
                if let ModuleItem::ModuleDecl(ModuleDecl::ExportDecl(e)) = decl {
                    if let Some(fndecl) = e.decl.as_fn_decl() {
                        let name = fndecl.ident.sym.as_str().to_string();
                        let mut params = vec![];
                        let mut results = vec![];
                        if let Some(return_type) = fndecl.function.clone().return_type.clone() {
                            result_type(&mut results, &return_type.type_ann)?;
                        }

                        for param in fndecl.function.params.iter() {
                            let name = param.pat.clone().expect_ident().id.sym.as_str().to_string();
                            let p = param.pat.clone().expect_ident();
                            match p.type_ann {
                                None => params.push(Param::new(&name, val_type("i64"))),
                                Some(ann) => {
                                    param_type(&mut params, &name, &ann.type_ann)?;
                                }
                            }
                        }
                        let signature = Signature {
                            name,
                            params,
                            results,
                        };

                        signatures.push(signature);
                    }
                } else {
                    bail!("Don't know what to do with non export on main module");
                }
            }
        }
    }

    Ok(Interface {
        name: "main".to_string(),
        functions: signatures,
    })
}

/// Parse the whole TS module type file
fn parse_module(module: Module) -> Result<Vec<Interface>> {
    let mut interfaces = Vec::new();
    for statement in &module.body {
        if let ModuleItem::Stmt(Stmt::Decl(Decl::TsModule(submod))) = statement {
            let name = submod.id.as_str().map(|name| name.value.as_str());

            match name {
                Some("main") | None => {
                    interfaces.push(parse_module_decl(submod)?);
                }
                Some(_) => {
                    if let Some(imports) = parse_imports(submod)? {
                        interfaces.push(imports);
                    }
                }
            };
        }
    }

    Ok(interfaces)
}

/// Parse the d.ts file representing the plugin interface
pub fn parse_interface_file(interface_path: &PathBuf) -> Result<PluginInterface> {
    let cm: Lrc<SourceMap> = Default::default();
    let fm = cm.load_file(interface_path)?;
    let lexer = Lexer::new(
        Syntax::Typescript(Default::default()),
        Default::default(),
        StringInput::from(&*fm),
        None,
    );

    let mut parser = Parser::new_from(lexer);
    let parse_errs = parser.take_errors();
    if !parse_errs.is_empty() {
        for e in parse_errs {
            log::error!("{:#?}", e);
        }
        bail!("Failed to parse TypeScript interface file. It is not valid TypeScript.");
    }

    let module = parser.parse_module().expect("failed to parser module");
    let interfaces = parse_module(module)?;
    let exports = interfaces
        .iter()
        .find(|i| i.name == "main")
        .context("You need to declare a 'main' module")?
        .to_owned();

    let imports = interfaces
        .into_iter()
        .filter(|i| i.name != "main")
        .collect();

    Ok(PluginInterface { exports, imports })
}
