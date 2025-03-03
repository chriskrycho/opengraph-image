use std::path::PathBuf;

use clap::Parser;

fn main() {
    let args = Args::parse();
    let image = opengraph_image::render(&args.page_title);
    std::fs::write(&args.out, image).unwrap_or_else(|cause| {
        panic!(
            "Could not write to '{}'. Cause: {cause}",
            args.out.display()
        )
    });
}

#[derive(Debug, Parser)]
#[command(arg_required_else_help = true)]
struct Args {
    /// The page title to render as an image.
    page_title: String,

    /// Where to emit the file
    out: PathBuf,
}
