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

    /// Query for finding the lump to read. The simplest query is just the
    /// lump name, for example "endoom".
    ///
    /// To locate a lump that comes after some given lump, use +, for example
    /// "e1m3+linedefs".
    ///
    /// To locate a lums that appears within a section delimited by *_START
    /// and *_END, use /. For example "f/step1".
    ///
    /// Matching is case insensitive.
    query: String,
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let opt = Opt::from_args();

    let wad = wad::load_wad_file(opt.input)?;

    let mut wad = wad.as_slice();
    let mut last = 0;
    for (index, op) in opt.query.match_indices(|c| c == '+' || c == '/') {
        let part = &opt.query[last..index];
        last = index + op.len();

        wad = match op {
            "+" => {
                let id = wad::EntryId::from_str(part)
                    .ok_or_else(|| format!("Invalid lump ID: {:?}", part))?;
                let index = wad
                    .index_of(id)
                    .ok_or_else(|| format!("Lump not found: {:?}", part))?;
                wad.slice(index..)
            },
            "/" => {
                let start_id = wad::EntryId::from_str(format!("{}_START", part))
                    .ok_or_else(|| format!("Invalid lump ID: {:?}_START", part))?;
                let start_index = wad
                    .index_of(start_id)
                    .ok_or_else(|| format!("Lump not found: {:?}_START", part))?;

                let end_id = wad::EntryId::from_str(format!("{}_END", part))
                    .ok_or_else(|| format!("Invalid lump ID: {:?}_END", part))?;
                let end_index = wad
                    .index_of(end_id)
                    .ok_or_else(|| format!("Lump not found: {:?}_END", part))?;

                wad.slice(start_index+1..end_index)
            },
            _ => unreachable!()
        };
    }

    let name = &opt.query[last..];
    let id = wad::EntryId::from_str(name).ok_or_else(|| format!("Invalid lump ID: {:?}", name))?;
    let index = wad.index_of(id).ok_or_else(|| format!("Lump not found: {:?}", name))?;

    std::io::stdout()
        .lock()
        .write_all(wad.entry(index)?.lump)?;

    Ok(())
}
