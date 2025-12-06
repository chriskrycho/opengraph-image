use std::sync::LazyLock;

use ril::{
    Font, HorizontalAnchor, ImageFormat, Rgb, TextAlign, TextLayout, TextSegment, WrapStyle,
};

pub struct Content {
    pub title: String,
    pub subtitle: Option<String>,
}

pub fn render(content: Content) -> Vec<u8> {
    println!("INFO: Rendering image:");
    println!("      title:   '{}'", content.title);
    println!(
        "      subitle: '{}'",
        content
            .subtitle
            .as_ref()
            .map(|s| s.as_str())
            .unwrap_or("(none)")
    );

    // In principle, this could be a slowdown if there is contention, but in
    // practice I benchmarked it and… there isn’t enough for it to matter; it
    // runs the same speed either way.
    let fonts = &*FONTS;

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
            &fonts.sanomat_sans_text_regular,
            content.title,
            POST_TITLE_COLOR,
        );

    let post_subtitle = content.subtitle.as_ref().map(|subtitle| {
        TextLayout::new()
            .with_wrap(WrapStyle::Word)
            .with_align(TextAlign::Left)
            .with_position(56, post_title.height() + 56)
            .with_width(1_416)
            .with_basic_text(
                &fonts.md_lorien_regular_italic,
                subtitle,
                POST_SUBTITLE_COLOR,
            )
    });

    println!("{:?}", &content.subtitle);

    let site_title = TextLayout::new()
        .with_horizontal_anchor(HorizontalAnchor::Right)
        .with_position(1_472, 527)
        .with_basic_text(&fonts.sanomat_semibold, "Sym·poly·mathesy", SITE_COLOR);

    let author = TextLayout::new()
        .with_horizontal_anchor(HorizontalAnchor::Right)
        .with_position(1_464, 640)
        .with_segment(&TextSegment::new(
            &fonts.md_lorien_book_italic,
            "by",
            AUTHOR_COLOR,
        ))
        .with_segment(&TextSegment::new(
            &fonts.md_lorien_book,
            " Chris Krycho",
            AUTHOR_COLOR,
        ));

    let image = ril::Image::new(1528, 800, IMAGE_BG)
        .with(&inset)
        .with(&author)
        .with(&site_title)
        .with(&post_title);

    let image = match post_subtitle {
        Some(subtitle) => image.with(&subtitle),
        None => image,
    };

    let mut data = Vec::<u8>::with_capacity(image.data.len());
    image.encode(ImageFormat::Png, &mut data).unwrap();
    data
}

const IMAGE_BG: ril::Rgb = ril::Rgb::new(241, 242, 244);
const TEXT_BG: ril::Rgb = ril::Rgb::new(252, 252, 253);
const BORDER: ril::Rgb = ril::Rgb::new(171, 175, 186);

const POST_TITLE_COLOR: Rgb = Rgb::new(34, 37, 42);
const POST_SUBTITLE_COLOR: Rgb = Rgb::new(80, 86, 98);
const SITE_COLOR: Rgb = Rgb::new(13, 89, 156);
const AUTHOR_COLOR: Rgb = Rgb::new(34, 37, 42);

const SANOMAT_SANS_TEXT_REGULAR: &[u8] = include_bytes!("../fonts/SanomatSansText-Regular.otf");
const SANOMAT_SEMIBOLD: &[u8] = include_bytes!("../fonts/Sanomat-Semibold.otf");
const MD_LORIEN_REGULAR_ITALIC: &[u8] = include_bytes!("../fonts/MDLórien-Italic.otf");
const MD_LORIEN_BOOK: &[u8] = include_bytes!("../fonts/MDLórien-Book.otf");
const MD_LORIEN_BOOK_ITALIC: &[u8] = include_bytes!("../fonts/MDLórien-BookItalic.otf");

struct Fonts {
    sanomat_sans_text_regular: Font,
    sanomat_semibold: Font,
    md_lorien_regular_italic: Font,
    md_lorien_book: Font,
    md_lorien_book_italic: Font,
}

static FONTS: LazyLock<Fonts> = LazyLock::new(|| Fonts {
    sanomat_sans_text_regular: Font::from_bytes(SANOMAT_SANS_TEXT_REGULAR, 68.0)
        .expect("could not load Sanomat Sans Text Book"),
    sanomat_semibold: Font::from_bytes(SANOMAT_SEMIBOLD, 120.0).expect("Sanomat Semibold"),
    md_lorien_regular_italic: Font::from_bytes(MD_LORIEN_REGULAR_ITALIC, 56.0)
        .expect("could not load MD Lórien Italic"),
    md_lorien_book: Font::from_bytes(MD_LORIEN_BOOK, 80.0).expect("could not load MD Lórien Book"),
    md_lorien_book_italic: Font::from_bytes(MD_LORIEN_BOOK_ITALIC, 80.0)
        .expect("could not load MD Lórien Book Italic"),
});
