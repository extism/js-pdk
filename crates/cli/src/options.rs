use std::path::PathBuf;
use structopt::StructOpt;

#[derive(Debug, StructOpt)]
#[structopt(name = "extism-js", about = "Extism JavaScript PDK Plugin Compiler")]
pub struct Options {
    #[structopt(parse(from_os_str))]
    pub input_js: PathBuf,

    #[structopt(short = "i", parse(from_os_str), default_value = "index.d.ts")]
    pub interface_file: PathBuf,

    #[structopt(short = "o", parse(from_os_str), default_value = "index.wasm")]
    pub output: PathBuf,
}
