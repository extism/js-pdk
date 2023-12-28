mod opt;
mod options;
mod shims;
mod ts_parser;

use crate::options::Options;
use crate::ts_parser::parse_interface_file;
use anyhow::{bail, Result};
use shims::generate_wasm_shims;
use std::env;
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

    // We need to parse the interface.d.ts file
    let interface_path = PathBuf::from(&opts.interface_file);
    if !interface_path.exists() {
        bail!(
            "Could not find interface file {}. Set to a valid d.ts file with the -i flag",
            &interface_path.to_str().unwrap()
        );
    }
    let plugin_interface = parse_interface_file(&interface_path)?;

    // Copy in the user's js code from stdin
    let mut input_file = fs::File::open(&opts.input_js)?;
    let mut user_code: Vec<u8> = vec![];
    input_file.read_to_end(&mut user_code)?;

    // If we have imports, we need to inject some state needed for host function support
    let names = &plugin_interface
        .imports
        .functions
        .iter()
        .map(|s| format!("'{}'", &s.name))
        .collect::<Vec<String>>()
        .join(",");
    let mut contents = format!("Host.__hostFunctions = [{}].sort();\n", names)
        .as_bytes()
        .to_owned();
    contents.append(&mut user_code);

    // Create a tmp dir to hold all the library objects
    // This can go away once we do all the wasm-merge stuff in process
    let tmp_dir = TempDir::new()?;
    let core_path = tmp_dir.path().join("core.wasm");
    let export_shim_path = tmp_dir.path().join("export-shim.wasm");
    let import_shim_path = tmp_dir.path().join("import-shim.wasm");
    let linked_shim_path = tmp_dir.path().join("linked.wasm");

    // let tmp_dir = "/tmp/derp";
    // let core_path = PathBuf::from("/tmp/derp/core.wasm");
    // let export_shim_path = PathBuf::from("/tmp/derp/export-shim.wasm");
    // let import_shim_path = PathBuf::from("/tmp/derp/import-shim.wasm");
    // let linked_shim_path = PathBuf::from("/tmp/derp/linked.wasm");

    // First wizen the core module
    let self_cmd = env::args().next().expect("Expected a command argument");
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

    // Create our shim files given our parsed TS module object
    // We have a shim file for exports and one optional one for imports
    generate_wasm_shims(
        plugin_interface.exports,
        &export_shim_path,
        plugin_interface.imports,
        &import_shim_path,
    )?;

    // Merge the export shim with the core module
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

    // // If there is no import shim, then there are no imports
    // // and we can copy this intermediate wasm module as the output and return.
    // // There is a probably a better way to signal this than just checking
    // // for the existence of the file.
    // if !&import_shim_path.exists() {
    //     fs::copy(&linked_shim_path, &opts.output)?;
    //     return Ok(());
    // }

    // Merge the import shim with the core+export (linked) module
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

    Ok(())
}
