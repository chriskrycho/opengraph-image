use std::{io, path::PathBuf};

use clap::Parser;

fn main() -> Result<(), io::Error> {
    let args = Args::parse();
    let image = opengraph_image::render(opengraph_image::Content {
        title: &args.title,
        subtitle: args.subtitle.as_ref().map(|s| s.as_str()),
    });
    std::fs::write(&args.out, image)
}

#[derive(Debug, Parser)]
#[command(arg_required_else_help = true)]
struct Args {
    /// The page title to use in the image.
    title: String,

    /// The pgae subtitle (if any) to use in the image.
    subtitle: Option<String>,

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
