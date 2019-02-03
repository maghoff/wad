extern crate wad;

use std::io::Write;
use std::path::PathBuf;

use structopt::StructOpt;

#[derive(Debug, StructOpt)]
#[structopt(
    name = "wad-read",
    about = "Read the contents of a lump in a WAD file and print it to STDOUT"
)]
struct Opt {
    /// Input WAD file
    #[structopt(parse(from_os_str))]
    input: PathBuf,

    /// Lump index to read and print to STDOUT
    lump: usize,
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let opt = Opt::from_args();

    let wad = wad::load_wad_file(opt.input)?;

    std::io::stdout()
        .lock()
        .write_all(wad.entry(opt.lump)?.lump)?;

    Ok(())
}
