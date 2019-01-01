extern crate wad;

use std::path::PathBuf;
use structopt::StructOpt;

#[derive(Debug, StructOpt)]
#[structopt(name = "wadls", about = "List the lumps in a WAD file")]
struct Opt {
    /// Input file
    #[structopt(parse(from_os_str))]
    input: PathBuf,
}

fn main() -> Result<(), wad::LoadError> {
    let opt = Opt::from_args();

    let wad = wad::load_wad_file(opt.input)?;

    for (i, (name, data)) in wad.iter().enumerate() {
        println!("{}\t{}\t{}", i, data.len(), name);
    }

    Ok(())
}
