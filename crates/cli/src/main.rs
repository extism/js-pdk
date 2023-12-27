mod interface_parser;
mod opt;
mod options;
mod shim;

use crate::interface_parser::parse_interface_file;
use crate::options::Options;
use anyhow::{bail, Result};
use shim::create_shims;
use std::env;
use std::fs::remove_dir_all;
use std::io::{Read, Write};
use std::path::PathBuf;
use std::process::Stdio;
use std::{fs, process::Command};
use structopt::StructOpt;
use tempfile::TempDir;

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

    let interface_path = PathBuf::from(&opts.interface_file);
    if !interface_path.exists() {
        bail!(
            "Could not find interface file {}. Set to a valid d.ts file with the -i flag",
            &interface_path.to_str().unwrap()
        );
    }

    let plugin_interface = parse_interface_file(&interface_path)?;

    let mut input_file = fs::File::open(&opts.input_js)?;
    let mut user_code: Vec<u8> = vec![];
    input_file.read_to_end(&mut user_code)?;

    let mut contents = "Host.__hostFunctions = ['myHostFunc1', 'myHostFunc2'].sort();\n"
        .as_bytes()
        .to_owned();

    contents.append(&mut user_code);

    let self_cmd = env::args().next().expect("Expected a command argument");
    //let tmp_dir = TempDir::new()?;
    // let core_path = tmp_dir.path().join("core.wasm");
    // let export_shim_path = tmp_dir.path().join("export-shim.wasm");
    // let import_shim_path = tmp_dir.path().join("import-shim.wasm");
    // let linked_shim_path = tmp_dir.path().join("linked.wasm");

    let tmp_dir = "/tmp/derp";
    let core_path = PathBuf::from("/tmp/derp/core.wasm");
    let export_shim_path = PathBuf::from("/tmp/derp/export-shim.wasm");
    let import_shim_path = PathBuf::from("/tmp/derp/import-shim.wasm");
    let linked_shim_path = PathBuf::from("/tmp/derp/linked.wasm");

    {
        env::set_var("EXTISM_WIZEN", "1");
        let mut command = Command::new(self_cmd)
            .arg(&opts.input_js)
            .arg("-o")
            .arg(&core_path)
            .stdin(Stdio::piped())
            .spawn()?;
        command
            .stdin
            .take()
            .expect("Expected to get writeable stdin")
            .write_all(&contents)?;
        let status = command.wait()?;
        if !status.success() {
            bail!("Couldn't create wasm from input");
        }
    }

    create_shims(&plugin_interface, &export_shim_path, &import_shim_path)?;

    let mut command = Command::new("wasm-merge")
        .arg(&core_path)
        .arg("coremod")
        .arg(&export_shim_path)
        .arg("codemod")
        .arg("-o")
        .arg(&linked_shim_path)
        .spawn()?;
    let status = command.wait()?;
    if !status.success() {
        bail!("wasm-merge failed. Couldn't merge export shim");
    }

    if !&import_shim_path.exists() {
        fs::copy(&linked_shim_path, &opts.output)?;
        //remove_dir_all(tmp_dir)?;
        return Ok(());
    }

    println!("merge imports now to {:#?}", &opts.output.to_str());

    let mut command = Command::new("wasm-merge")
        .arg(&linked_shim_path)
        .arg("coremod")
        .arg(&import_shim_path)
        .arg("codemod")
        .arg("-o")
        .arg(&opts.output)
        .arg("--enable-reference-types")
        .arg("--enable-bulk-memory")
        .spawn()?;
    let status = command.wait()?;
    if !status.success() {
        bail!("wasm-merge failed. Couldn't merge import shim.");
    }

    //remove_dir_all(tmp_dir)?;

    Ok(())
}
