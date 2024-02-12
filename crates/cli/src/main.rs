mod opt;
mod options;
mod shims;
mod ts_parser;

use crate::options::Options;
use crate::ts_parser::parse_interface_file;
use anyhow::{bail, Result};
use env_logger::{Builder, Target};
use log::LevelFilter;
use shims::generate_wasm_shims;
use std::env;
use std::io::{Read, Write};
use std::path::PathBuf;
use std::process::Stdio;
use std::{fs, process::Command};
use structopt::StructOpt;
use tempfile::TempDir;

fn main() -> Result<()> {
    let mut builder = Builder::new();
    builder
        .filter(None, LevelFilter::Info)
        .target(Target::Stdout)
        .init();

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
    let mut contents = Vec::new();
    for ns in &plugin_interface.imports {
        let names = &ns
            .functions
            .iter()
            .map(|s| format!("'{}'", &s.name))
            .collect::<Vec<String>>()
            .join(",");
        contents
            .extend_from_slice(format!("Host.__hostFunctions = [{}].sort();\n", names).as_bytes());
    }
    contents.append(&mut user_code);

    // Create a tmp dir to hold all the library objects
    // This can go away once we do all the wasm-merge stuff in process
    let tmp_dir = TempDir::new()?;
    let core_path = tmp_dir.path().join("core.wasm");
    let shim_path = tmp_dir.path().join("shim.wasm");

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

    // Create our shim file given our parsed TS module object
    generate_wasm_shims(
        &shim_path,
        &plugin_interface.exports,
        &plugin_interface.imports,
    )?;

    let output = Command::new("wasm-merge").arg("--version").output();
    if let Err(_) = output {
        bail!("Failed to execute wasm-merge. Please install binaryen and make sure wasm-merge is on your path: https://github.com/WebAssembly/binaryen");
    }

    // Merge the shim with the core module
    let mut command = Command::new("wasm-merge")
        .arg(&core_path)
        .arg("core")
        .arg(&shim_path)
        .arg("shim")
        .arg("-o")
        .arg(&opts.output)
        .arg("--enable-reference-types")
        .arg("--enable-bulk-memory")
        .spawn()?;
    let status = command.wait()?;
    if !status.success() {
        bail!("wasm-merge failed. Couldn't merge shim");
    }

    Ok(())
}
