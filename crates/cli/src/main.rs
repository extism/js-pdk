mod opt;
mod options;
mod shim;

use crate::options::Options;
use anyhow::{bail, Result};
use shim::create_shims;
use std::env;
use std::io::{Read, Write};
use std::path::PathBuf;
use std::process::Stdio;
use std::{fs, process::Command};
use structopt::StructOpt;

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

    let mut input_file = fs::File::open(&opts.input)?;
    let mut contents: Vec<u8> = vec![];
    input_file.read_to_end(&mut contents)?;

    let self_cmd = env::args().next().unwrap();
    let core = "core.wasm";

    {
        env::set_var("EXTISM_WIZEN", "1");
        let mut command = Command::new(self_cmd)
            .arg(&opts.input)
            .arg("-o")
            .arg(core)
            .stdin(Stdio::piped())
            .spawn()?;
        command.stdin.take().unwrap().write_all(&contents)?;
        let status = command.wait()?;
        if !status.success() {
            bail!("Couldn't create wasm from input");
        }
    }

    let interface_path = PathBuf::from(&opts.interface);
    let export_shim_path = PathBuf::from("export-shim.wasm");
    create_shims(interface_path, export_shim_path)?;

    let mut command = Command::new("wasm-merge")
        .arg(core)
        .arg("coremod")
        .arg("export-shim.wasm")
        .arg("codemod")
        .arg("-o")
        .arg(&opts.output)
        .spawn()?;
    let status = command.wait()?;
    if !status.success() {
        bail!("Couldn't run wasm-merge");
    }

    Ok(())
}
