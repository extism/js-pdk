use std::path::PathBuf;
use structopt::StructOpt;

#[derive(Debug, StructOpt)]
#[structopt(name = "extism-js", about = "Extism JavaScript PDK Plugin Compiler")]
pub struct Options {
    #[structopt(
        parse(from_os_str),
        about = "Input JS program for the plugin. Needs to be a single file."
    )]
    pub input_js: PathBuf,

    #[structopt(
        short = "i",
        parse(from_os_str),
        default_value = "index.d.ts",
        about = "d.ts file describing the plug-in interface."
    )]
    pub interface_file: PathBuf,

    #[structopt(
        short = "o",
        parse(from_os_str),
        default_value = "index.wasm",
        about = "Ouput wasm file."
    )]
    pub output: PathBuf,

    #[structopt(short = "c", about = "Include the core")]
    pub core: bool,

    #[structopt(long = "--skip-opt", about = "Skip final optimization pass")]
    pub skip_opt: bool,
}
