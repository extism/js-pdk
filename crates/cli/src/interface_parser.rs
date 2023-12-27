extern crate swc_common;
extern crate swc_ecma_parser;
use anyhow::{bail, Context, Result};
use std::fs::File;
use std::io::prelude::*;
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
    pub imports: Option<Interface>,
}

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
                                Param {
                                    name: vn.to_string(),
                                    ptype: typ.to_string(),
                                }
                            })
                            .collect::<Vec<Param>>();
                        let results = Vec::new();
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
                        let results = vec![Param {
                            name: "result".to_string(),
                            ptype: return_type.to_string(),
                        }];

                        if !params.is_empty() {
                            bail!("An Extism export should take no params and return I32")
                        }
                        if results.len() != 1 {
                            bail!("An Extism export should return an I32")
                        }
                        let return_type = &results.get(0).unwrap().ptype;
                        if return_type != "I32" {
                            bail!("An Extism export should return an I32 not {}", return_type)
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
            log::warn!("{:#?}", e);
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
        .map(|i| i.to_owned());

    Ok(PluginInterface { exports, imports })
}
