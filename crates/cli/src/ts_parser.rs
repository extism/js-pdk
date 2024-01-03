extern crate swc_common;
extern crate swc_ecma_parser;
use anyhow::{bail, Context, Result};
use std::path::PathBuf;

use swc_common::sync::Lrc;
use swc_common::SourceMap;
use swc_ecma_ast::{Decl, Module, ModuleDecl, Stmt, TsInterfaceDecl, TsModuleDecl};
use swc_ecma_ast::{ModuleItem, TsTypeElement};
use swc_ecma_parser::{lexer::Lexer, Parser, StringInput, Syntax};

#[derive(Debug, Clone)]
pub struct Param {
    pub name: String,
    pub ptype: String,
}

impl Param {
    pub fn new(name: &str, ptype: &str) -> Param {
        Param {
            name: name.to_string(),
            ptype: ptype.to_string().to_uppercase(),
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
    pub imports: Interface,
}

/// Parses the "user" part of the module which maps to the wasm imports
fn parse_user_interface(i: &Box<TsInterfaceDecl>) -> Result<Option<Interface>> {
    let mut signatures = Vec::new();
    let name = i.id.sym.as_str();
    match name {
        "user" => {
            for sig in &i.body.body {
                match sig {
                    TsTypeElement::TsMethodSignature(t) => {
                        let name = t.key.as_ident().unwrap().sym.to_string();
                        let params = t
                            .params
                            .iter()
                            .map(|p| {
                                let vn = p.as_ident().unwrap().id.sym.as_str();
                                let typ = p.as_ident().unwrap().type_ann.clone();
                                let typ = typ.unwrap();
                                let typ = &typ
                                    .type_ann
                                    .as_ts_type_ref()
                                    .unwrap()
                                    .type_name
                                    .as_ident()
                                    .unwrap()
                                    .sym;
                                Param::new(vn, typ)
                            })
                            .collect::<Vec<Param>>();
                        let return_type = &t.type_ann.clone().context("Missing return type")?;
                        let return_type = &return_type
                            .type_ann
                            .as_ts_type_ref()
                            .context("Illegal return type")?
                            .type_name
                            .as_ident()
                            .context("Illegal return type")?
                            .sym;
                        let results = vec![Param::new("return", return_type)];
                        let signature = Signature {
                            name,
                            params,
                            results,
                        };
                        signatures.push(signature);
                    }
                    _ => {
                        println!("Warning: don't know what to do with sig {:#?}", sig);
                    }
                }
            }

            Ok(Some(Interface {
                name: name.into(),
                functions: signatures,
            }))
        }
        _ => Ok(None),
    }
}

/// Try to parse the imports
fn parse_imports(tsmod: &Box<TsModuleDecl>) -> Result<Option<Interface>> {
    for block in &tsmod.body {
        if let Some(block) = block.clone().ts_module_block() {
            for inter in block.body {
                if let ModuleItem::Stmt(Stmt::Decl(decl)) = inter {
                    let i = decl.as_ts_interface().unwrap();
                    let interface = parse_user_interface(i)?;
                    return Ok(interface);
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
                        let params = vec![]; // TODO ignoring params for now
                        let return_type = &fndecl
                            .function
                            .clone()
                            .return_type
                            .context("Missing return type")?
                            .clone();
                        let return_type = &return_type
                            .type_ann
                            .as_ts_type_ref()
                            .context("Illegal return type")?
                            .type_name
                            .as_ident()
                            .context("Illegal return type")?
                            .sym;
                        let results = vec![Param::new("result", return_type)];
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
            let name = if let Some(name) = submod.id.as_str() {
                Some(name.value.as_str())
            } else {
                None
            };

            match name {
                Some("extism:host") => {
                    if let Some(imports) = parse_imports(submod)? {
                        interfaces.push(imports);
                    }
                }
                Some("main") => {
                    interfaces.push(parse_module_decl(submod)?);
                }
                _ => {
                    log::warn!("Could not parse module with name {:#?}", name);
                }
            };
        }
    }

    Ok(interfaces)
}

fn validate_interface(plugin_interface: &PluginInterface) -> Result<()> {
    let mut has_err = false;
    let mut log_err = |msg: String| {
        println!("{}", msg);
        has_err = true;
    };

    for e in &plugin_interface.exports.functions {
        if !e.params.is_empty() {
            log_err(format!("The export {} should take no params", e.name));
        }
        if e.results.len() != 1 {
            log_err(format!("The export {} should return a single I32", e.name));
        } else {
            let return_type = &e.results.get(0).unwrap().ptype;
            if return_type != "I32" {
                log_err(format!(
                    "The export {} should return an I32 not {}",
                    e.name, return_type
                ));
            }
        }
    }

    for i in &plugin_interface.imports.functions {
        if i.results.is_empty() {
            log_err(format!("Import function {} needs to return an I64", i.name));
        } else if i.results.len() > 1 {
            log_err(format!(
                "Import function {} has too many returns. We only support 1 at the moment",
                i.name
            ));
        } else {
            let result = i.results.get(0).unwrap();
            if result.ptype != "I64" {
                log_err(format!(
                    "Import function {} needs to return an I64 but instead returns {}",
                    i.name, result.ptype
                ));
            }
        }

        if i.params.is_empty() {
            log_err(format!(
                "Import function {} needs to accept a single I64 pointer as param",
                i.name
            ));
        } else if i.params.len() > 1 {
            log_err(format!(
                "Import function {} has too many params. We only support 1 at the moment",
                i.name
            ));
        } else {
            let param = i.params.get(0).unwrap();
            if param.ptype != "I64" {
                log_err(format!(
                    "Import function {} needs to accept a single I64 but instead takes {}",
                    i.name, param.ptype
                ));
            }
        }
    }

    if has_err {
        bail!("Failed to validate plugin interface file");
    }

    Ok(())
}

/// Parse the d.ts file representing the plugin interface
pub fn parse_interface_file(interface_path: &PathBuf) -> Result<PluginInterface> {
    let cm: Lrc<SourceMap> = Default::default();
    let fm = cm.load_file(&interface_path)?;
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
        bail!("Failed to parse typescript interface file.");
    }

    let module = parser.parse_module().expect("failed to parser module");
    let interfaces = parse_module(module)?;
    let exports = interfaces
        .iter()
        .find(|i| i.name == "main")
        .context("You need to declare a 'main' module")?
        .to_owned();
    let imports = interfaces
        .iter()
        .find(|i| i.name == "user")
        .map(|i| i.to_owned())
        .unwrap_or(Interface {
            name: "user".into(),
            functions: vec![],
        });

    let plugin_interface = PluginInterface { exports, imports };
    validate_interface(&plugin_interface)?;
    Ok(plugin_interface)
}
