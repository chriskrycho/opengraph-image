use std::{io, path::PathBuf};

use clap::Parser;

fn main() -> Result<(), io::Error> {
    let args = Args::parse();
    let image = opengraph_image::render(&args.page_title);
    std::fs::write(&args.out, image)
}

#[derive(Debug, Parser)]
#[command(arg_required_else_help = true)]
struct Args {
    /// The page title to render as an image.
    page_title: String,

    /// Where to emit the file
    #[arg(short, long)]
    out: PathBuf,

    /// Which site to build for. Defaults to Sympolymathesy if not specified.
    #[arg(short, long)]
    site: Site,
}

#[derive(Debug, Default, Clone, clap::ValueEnum)]
enum Site {
    #[default]
    Sympolymathesy,
    Music,
}
