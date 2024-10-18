mod opt;
mod options;
mod shims;
mod ts_parser;

use crate::options::Options;
use crate::ts_parser::parse_interface_file;
use anyhow::{bail, Result};
use log::LevelFilter;
use shims::generate_wasm_shims;
use std::env;
use std::path::PathBuf;
use std::process::Stdio;
use std::{fs, io::Write, process::Command};
use structopt::StructOpt;
use tempfile::TempDir;

const CORE: &[u8] = include_bytes!(concat!(env!("OUT_DIR"), "/engine.wasm"));

fn main() -> Result<()> {
    let mut builder = env_logger::Builder::new();
    builder
        .filter(None, LevelFilter::Info)
        .target(env_logger::Target::Stdout)
        .init();

    let opts = Options::from_args();

    if opts.core {
        opt::Optimizer::new(CORE)
            .wizen(true)
            .write_optimized_wasm(opts.output)?;
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

    // Copy in the user's js code from the configured file
    let mut user_code = fs::read(&opts.input_js)?;

    // If we have imports, we need to inject some state needed for host function support
    let mut contents = Vec::new();
    let mut names = Vec::new();
    let mut sorted_names = Vec::new();
    for ns in &plugin_interface.imports {
        sorted_names.extend(ns.functions.iter().map(|s| (&s.name, s.results.len())));
    }
    sorted_names.sort_by_key(|x| x.0.as_str());

    for (name, results) in sorted_names {
        names.push(format!("{{ name: '{}', results: {} }}", &name, results));
    }

    contents
        .extend_from_slice(format!("Host.__hostFunctions = [{}];\n", names.join(", ")).as_bytes());
    contents.append(&mut user_code);

    // Create a tmp dir to hold all the library objects
    // This can go away once we do all the wasm-merge stuff in process
    let tmp_dir = TempDir::new()?;
    let core_path = tmp_dir.path().join("core.wasm");
    let shim_path = tmp_dir.path().join("shim.wasm");

    // First wizen the core module
    let self_cmd = env::args().next().expect("Expected a command argument");
    {
        let mut command = Command::new(self_cmd)
            .arg("-c")
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

    let output = Command::new("wasm-merge")
        .arg("--version")
        .stdout(Stdio::null())
        .stderr(Stdio::null())
        .status();
    if output.is_err() {
        bail!("Failed to detect wasm-merge. Please install binaryen and make sure wasm-merge is on your path: https://github.com/WebAssembly/binaryen");
    }

    // Merge the shim with the core module
    let status = Command::new("wasm-merge")
        .arg(&core_path)
        .arg("core")
        .arg(&shim_path)
        .arg("shim")
        .arg("-o")
        .arg(&opts.output)
        .arg("--enable-reference-types")
        .arg("--enable-bulk-memory")
        .status()?;
    if !status.success() {
        bail!("wasm-merge failed. Couldn't merge shim");
    }

    if !opts.skip_opt {
        opt::optimize_wasm_file(opts.output)?;
    }

    Ok(())
}
