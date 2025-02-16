use ril::Font;

fn main() {
    println!("Hello, world!");
}

const SANOMAT_SANS_TEXT_SEMIBOLD_ITALIC: &[u8] =
    include_bytes!("../fonts/SanomatSansText-SemiboldItalic.otf");
const SANOMAT_SEMIBOLD: &[u8] = include_bytes!("../fonts/Sanomat-Semibold.otf");
const FRAME_HEAD: &[u8] = include_bytes!("../fonts/FrameHead-Roman.otf");
const FRAME_HEAD_ITALIC: &[u8] = include_bytes!("../fonts/FrameHead-Italic.otf");

fn load_fonts() -> Vec<Font> {
    vec![
        Font::from_bytes(SANOMAT_SANS_TEXT_SEMIBOLD_ITALIC, 90.0)
            .expect("could not load Sanomat Sans Text Semibold Italic"),
        Font::from_bytes(SANOMAT_SEMIBOLD, 132.0).expect("Sanomat Semibold"),
        Font::from_bytes(FRAME_HEAD, 90.0).expect("could not load Frame Head"),
        Font::from_bytes(FRAME_HEAD_ITALIC, 90.0).expect("could not load Frame Head Italic"),
    ]
}
