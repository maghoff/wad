extern crate wad;

use std::path::PathBuf;

use structopt::StructOpt;

#[derive(Debug, StructOpt)]
#[structopt(name = "wad-ls", about = "List the lumps in a WAD file")]
struct Opt {
    /// Input WAD file
    #[structopt(parse(from_os_str))]
    input: PathBuf,
}

fn main() -> Result<(), Box<std::error::Error>> {
    let opt = Opt::from_args();

    let wad = wad::load_wad_file(opt.input)?;

    for (i, lump) in wad.iter().enumerate() {
        println!("{}\t{}\t{}", i, lump.data.len(), lump.name()?);
    }

    Ok(())
}
