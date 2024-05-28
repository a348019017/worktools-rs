use clap::builder::Str;
use clap::{Error, Parser};
use serde_json::Number;
use std::fs::{self, create_dir_all};
use std::path::{Path, PathBuf};
use std::time::{Duration, Instant};
//use error_chain::ChainedError;
use glob::{glob_with, MatchOptions};
use rayon::prelude::*;
use serde::{Deserialize, Serialize};

#[derive(Parser, Debug, Serialize, Deserialize)]
#[command(version, about, long_about = None)]
struct SingleArgs {
    /// 输出路径
    #[arg(short, long)]
    output: Option<std::path::PathBuf>,
    /// 输入路径，注意文件夹的结构，路径中需至少包含一个子文件夹用于分组
    #[arg(short, long)]
    input: Option<std::path::PathBuf>,
}

pub fn parse_obj_entry() -> Result<(), Box<dyn std::error::Error>> {
    let args = Cli::parse();

    let mut options: MatchOptions = Default::default();
    options.case_sensitive = false;

    let mut _default_inputpath = PathBuf::new();
    if let Some(input) = args.input {
        _default_inputpath = input;
    }
    let mut _default_outputpath = PathBuf::new();
    if let Some(output) = args.output {
        _default_outputpath = output;
    }

    //
    let files: Vec<_> = glob_with(cur_path.to_str().unwrap(), options)?
        .filter_map(|x: Result<std::path::PathBuf, glob::GlobError>| x.ok())
        .collect();

    Ok(())
}
