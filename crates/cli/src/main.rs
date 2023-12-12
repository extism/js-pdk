mod opt;
mod options;
mod shim;

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

    let mut input_file = fs::File::open(&opts.input_js)?;
    let mut contents: Vec<u8> = vec![];
    input_file.read_to_end(&mut contents)?;

    let self_cmd = env::args().next().expect("Expected a command argument");
    let tmp_dir = TempDir::new()?;
    let core_path = tmp_dir.path().join("core.wasm");
    let export_shim_path = tmp_dir.path().join("export-shim.wasm");

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

    let interface_path = PathBuf::from(&opts.interface_file);
    create_shims(&interface_path, &export_shim_path)?;

    let mut command = Command::new("wasm-merge")
        .arg(&core_path)
        .arg("coremod")
        .arg(&export_shim_path)
        .arg("codemod")
        .arg("-o")
        .arg(&opts.output)
        .spawn()?;
    let status = command.wait()?;
    if !status.success() {
        bail!("Couldn't run wasm-merge");
    }

    remove_dir_all(tmp_dir)?;

    Ok(())
}
