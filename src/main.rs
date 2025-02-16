mod b2;

use std::io;

use reqwest::Client;
use ril::{Font, Image, ImageFormat, Rgb, TextAlign, TextLayout, TextSegment, WrapStyle};

#[tokio::main(flavor = "current_thread")]
async fn main() -> Result<(), Error> {
    let img = render("Tidy First? A Personal Exercise in Empirical Software Design");
    let file_name = "testing.png";
    let mut data = Vec::<u8>::with_capacity(img.data.len());
    img.encode(ImageFormat::Png, &mut data).unwrap();

    let auth_file = std::fs::read_to_string("Secrets.toml").map_err(|source| Error::Io {
        message: String::from("Could not read Secrets.toml"),
        source,
    })?;
    let auth: Auth = toml::from_str(&auth_file)?;

    b2::ClientBuilder::new(auth.id, auth.key)
        .authorize(Client::new())
        .await?
        .upload_file(file_name, data)
        .await?;

    Ok(())
}

#[derive(Debug, serde::Deserialize)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
struct Auth {
    id: String,
    key: String,
}

#[derive(Debug, thiserror::Error)]
enum Error {
    #[error(transparent)]
    B2 {
        #[from]
        source: b2::Error,
    },

    #[error("{message}: {source}")]
    Io { message: String, source: io::Error },

    #[error("Could not deserialize secrets")]
    DeserializeSecrets {
        #[from]
        source: toml::de::Error,
    },
}

fn render(text: &str) -> Image<Rgb> {
    let fonts = Fonts::load();

    // The offset *would* be 32, 32, but needs to take off one pixel on each
    // side to account for the extra pixel outside the inset rectangle one each
    // side courtesy of the 3px centered border.
    let inset = ril::Rectangle::at(32, 32)
        .with_border(ril::Border {
            color: BORDER,
            thickness: 3,
            position: ril::BorderPosition::Center,
        })
        .with_size(1464, 736)
        .with_fill(TEXT_BG);

    let post_title = TextLayout::new()
        .with_wrap(WrapStyle::Word)
        .with_align(TextAlign::Left)
        .with_position(56, 56)
        .with_width(1_416)
        .with_basic_text(
            &fonts.sanomat_sans_text_semibold_italic,
            text,
            POST_TITLE_COLOR,
        );

    let site_title = TextSegment::new(&fonts.sanomat_semibold, "Sym·poly·mathesy", SITE_COLOR)
        .with_position(56, 527);

    let author = TextLayout::new()
        .with_align(TextAlign::Left)
        .with_position(795, 662)
        .with_segment(&TextSegment::new(
            &fonts.frame_head_italic,
            "by",
            AUTHOR_COLOR,
        ))
        .with_segment(&TextSegment::new(
            &fonts.frame_head,
            " Chris Krycho",
            AUTHOR_COLOR,
        ));

    ril::Image::new(1528, 800, IMAGE_BG)
        .with(&inset)
        .with(&author)
        .with(&site_title)
        .with(&post_title)
}

const IMAGE_BG: ril::Rgb = ril::Rgb::new(241, 242, 244);
const TEXT_BG: ril::Rgb = ril::Rgb::new(252, 252, 253);
const BORDER: ril::Rgb = ril::Rgb::new(171, 175, 186);

const POST_TITLE_COLOR: Rgb = Rgb::new(34, 37, 42);
const SITE_COLOR: Rgb = Rgb::new(13, 89, 156);
const AUTHOR_COLOR: Rgb = Rgb::new(34, 37, 42);

const SANOMAT_SANS_TEXT_SEMIBOLD_ITALIC: &[u8] =
    include_bytes!("../fonts/SanomatSansText-SemiboldItalic.otf");
const SANOMAT_SEMIBOLD: &[u8] = include_bytes!("../fonts/Sanomat-Semibold.otf");
const FRAME_HEAD: &[u8] = include_bytes!("../fonts/FrameHead-Roman.otf");
const FRAME_HEAD_ITALIC: &[u8] = include_bytes!("../fonts/FrameHead-Italic.otf");

struct Fonts {
    sanomat_sans_text_semibold_italic: Font,
    sanomat_semibold: Font,
    frame_head: Font,
    frame_head_italic: Font,
}

impl Fonts {
    fn load() -> Fonts {
        Fonts {
            sanomat_sans_text_semibold_italic: Font::from_bytes(
                SANOMAT_SANS_TEXT_SEMIBOLD_ITALIC,
                90.0,
            )
            .expect("could not load Sanomat Sans Text Semibold Italic"),
            sanomat_semibold: Font::from_bytes(SANOMAT_SEMIBOLD, 132.0).expect("Sanomat Semibold"),
            frame_head: Font::from_bytes(FRAME_HEAD, 90.0).expect("could not load Frame Head"),
            frame_head_italic: Font::from_bytes(FRAME_HEAD_ITALIC, 90.0)
                .expect("could not load Frame Head Italic"),
        }
    }
}
