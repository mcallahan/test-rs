use anyhow::{Context, Result};
use clap::Parser;
use memmap::MmapOptions;
use rayon::prelude::*;
use std::fs::{File, OpenOptions};
use std::io::{Seek, SeekFrom, Write};

/// Fix a json file by quickly converting all commas into colons.
#[derive(Parser, Debug)]
#[command(author, version, about, long_about = None)]
struct Args {
    /// Input json file
    #[arg(short, long)]
    input: String,

    /// Output json file
    #[arg(short, long)]
    output: String,
}

fn main() -> Result<()> {
    let args = Args::parse();

    // Memmap the input file for reading.
    let ifile = File::open(&args.input)
        .with_context(|| format!("Unable to open input file '{}'", &args.input))?;
    let idata = unsafe {
        MmapOptions::new()
            .map(&ifile)
            .expect("Unable to mmap input file")
    };

    let len = idata.len();

    // Create the output file
    let mut ofile = OpenOptions::new()
        .read(true)
        .write(true)
        .create(true)
        .open(&args.output)
        .with_context(|| format!("Unable to open output file '{}'", &args.output))?;

    // Force the output file to be the same size as the input file.
    // Do this before mmap so that the space exists.
    // This assumes file sizes that fit in usize
    // (64 bit machines or files under 4GB).
    ofile
        .seek(SeekFrom::Start((len - 1) as u64))
        .expect("Unable to seek to end of output file");
    ofile
        .write_all(&[0])
        .expect("Unable to write end of output file. FS full?");
    ofile
        .seek(SeekFrom::Start(0))
        .expect("Unable to seek to start of output file");

    // Memmap the output file for writing.
    let mut odata = unsafe {
        memmap::MmapOptions::new()
            .map_mut(&ofile)
            .expect("Unable to mmap output file")
    };

    // Parallel iteration over the output and input buffers
    // while converting any ';' into ':'.
    //
    // Because both ';' and ':' are ASCII characters this works
    // on UTF-8 files as there will be no byte overlaps between
    // ASCII characters and UTF-8 characters.  Working in byte space
    // here saves considerable time.
    (&mut odata[..], &idata[..])
        .into_par_iter()
        .for_each(|(out, i)| *out = if *i == b';' { b':' } else { *i });

    Ok(())
}
