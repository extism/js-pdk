mod options;
mod opt;

use crate::options::Options;
use anyhow::{bail, Result};
use std::env;
use std::io::{Read, Write};
use std::path::Path;
use std::process::Stdio;
use std::{fs, process::Command};
use structopt::StructOpt;

fn main() -> Result<()> {
    let opts = Options::from_args();
    let wizen = env::var("EXTISM_WIZEN");

    if wizen.eq(&Ok("1".into())) {
        //let wasm: &[u8] = include_bytes!(concat!(env!("OUT_DIR"), "/engine.wasm"));
        // you probably just came here because you got an error, sorry :)
        // good on you for looking at the code before you came to yell at me.
        // change this to absolute path on your machine and it should work
        let wasm: &[u8] = include_bytes!("/Users/ben/Code/dylibso/js-pdk/target/wasm32-wasi/release/js_pdk_core.wasm");
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

    {
        env::set_var("EXTISM_WIZEN", "1");
        let mut command = Command::new(self_cmd)
            .arg(&opts.input)
            .arg("-o")
            .arg(&opts.output)
            .stdin(Stdio::piped())
            .spawn()?;
        command.stdin.take().unwrap().write_all(&contents)?;
        let status = command.wait()?;
        if !status.success() {
            bail!("Couldn't create wasm from input");
        }
    }

    add_custom_section(&opts.output, "extism_source".to_string(), contents)?;

    Ok(())
}

fn add_custom_section<P: AsRef<Path>>(file: P, section: String, contents: Vec<u8>) -> Result<()> {
    use parity_wasm::elements::*;

    let mut compressed: Vec<u8> = vec![];
    brotli::enc::BrotliCompress(
        &mut std::io::Cursor::new(contents),
        &mut compressed,
        &brotli::enc::BrotliEncoderParams {
            quality: 11,
            ..Default::default()
        },
    )?;

    let mut module = parity_wasm::deserialize_file(&file)?;
    module
        .sections_mut()
        .push(Section::Custom(CustomSection::new(section, compressed)));
    parity_wasm::serialize_to_file(&file, module)?;

    Ok(())
}