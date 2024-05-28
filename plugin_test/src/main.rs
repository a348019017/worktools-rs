mod mymod;
use clap::{Error, Parser};
use mymod::SingleArgs;
//测试多线程导出操作，还多线程个毛线，debug是release的n倍
fn main() -> Result<(), Box<dyn std::error::Error>> {
    let args = SingleArgs::parse();
    let _ = mymod::obj_single_parser(args);
    Ok(())
}
