mod mymod;
use clap::{Error, Parser};
use mymod::SingleArgs;

#[no_mangle]
pub extern "C" fn execute(options: &str) -> Result<(), Box<dyn std::error::Error>> {
    let args: SingleArgs = serde_json::from_str(options)?;
    let _ = mymod::obj_single_parser(args);
    Ok(())
}
