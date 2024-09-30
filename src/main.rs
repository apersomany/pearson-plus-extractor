use std::{
    fs::File,
    io::{BufWriter, Cursor, Write},
    path::PathBuf,
};

use anyhow::Result;
use clap::Parser;
use printpdf::{
    image_crate::{codecs::png::PngDecoder, ImageDecoder},
    BuiltinFont, Image, ImageTransform, Mm, PdfDocument, TextMatrix, TextRenderingMode,
};
use reqwest::{
    header::{HeaderMap, COOKIE, REFERER},
    Client,
};
use serde::{de::Error, Deserializer};
use sonic_rs::Deserialize;
use tokio::join;

#[derive(Deserialize)]
struct Annotation {
    #[serde(
        rename = "TextPageData",
        deserialize_with = "deserialize_text_page_data"
    )]
    data: TextPageData,
}

fn deserialize_text_page_data<'de, D>(deserializer: D) -> Result<TextPageData, D::Error>
where
    D: Deserializer<'de>,
    D::Error: Error,
{
    let text_page_data = String::deserialize(deserializer)?;
    Ok(sonic_rs::from_str(&text_page_data).map_err(|error| D::Error::custom(error))?)
}

#[derive(Deserialize)]
struct TextPageData {
    #[serde(rename = "texts")]
    data: Vec<Text>,
}

#[derive(Deserialize)]
struct Text {
    #[serde(rename = "mt")]
    matrix: [f32; 6],
    #[serde(rename = "cs")]
    stream: Vec<(f32, f32, f32, f32, u32)>,
}

struct Extractor {
    client: Client,
}

impl Extractor {
    pub fn new(cookie: impl AsRef<str>, auth_token: impl AsRef<str>) -> Result<Self> {
        let mut default_headers = HeaderMap::new();
        default_headers.insert(REFERER, "https://plus.pearson.com/".parse()?);
        default_headers.insert(COOKIE, cookie.as_ref().parse()?);
        default_headers.insert("X-Authorization", auth_token.as_ref().parse()?);
        let client = Client::builder().default_headers(default_headers).build()?;
        Ok(Self { client })
    }

    pub async fn run(
        self,
        product_id: u32,
        uuid: impl AsRef<str>,
        output: impl Write,
    ) -> Result<()> {
        let image = self.get_image(product_id, uuid.as_ref(), 0).await?;
        let title = "Pearson Plus";
        let image = PngDecoder::new(Cursor::new(image)).unwrap();
        let (w, h) = image.dimensions();
        let (w, h) = (Mm(w as f32 / 12.0), Mm(h as f32 / 12.0));
        let (document, page, layer) = PdfDocument::new(title, w, h, "layer");
        let image_transform = ImageTransform {
            dpi: Some(300.0),
            ..Default::default()
        };
        let font = &document.add_builtin_font(BuiltinFont::TimesRoman).unwrap();
        let layer = document.get_page(page).get_layer(layer);
        let image = Image::try_from(image).unwrap();
        image.add_to_layer(layer, image_transform);
        for i in 1..u32::MAX {
            println!("Downloaded page {:04}.", i);
            let (image, texts) = join!(
                self.get_image(product_id, uuid.as_ref(), i),
                self.get_texts(product_id, uuid.as_ref(), i)
            );
            if let Ok(image) = PngDecoder::new(Cursor::new(image?)) {
                let (w, h) = image.dimensions();
                let (w, h) = (Mm(w as f32 / 12.0), Mm(h as f32 / 12.0));
                let (page, layer) = document.add_page(w, h, "layer");
                let layer = document.get_page(page).get_layer(layer);
                let image = Image::try_from(image)?;
                image.add_to_layer(layer.clone(), image_transform);
                layer.begin_text_section();
                layer.set_font(font, 1.0);
                layer.set_text_rendering_mode(TextRenderingMode::Invisible);
                for data in texts?.data {
                    let mut matrix = data.matrix;
                    for (x, y, _, _, char) in data.stream {
                        matrix[4] = x;
                        matrix[5] = y;
                        layer.set_text_matrix(TextMatrix::Raw(matrix));
                        if let Some(char) = char::from_u32(char) {
                            layer.write_text(char, font);
                        }
                    }
                }
                layer.end_text_section();
            } else {
                break;
            }
        }
        println!("Saving the document. This make take a while.");
        document.save(&mut BufWriter::new(output))?;
        Ok(())
    }

    async fn get_image(&self, product_id: u32, uuid: &str, page: u32) -> Result<Vec<u8>> {
        let dest = format!(
            "https://plus.pearson.com/eplayer/pdfassets/prod1/{product_id}/{uuid}/pages/page{page}"
        );
        let resp = self.client.get(dest).send().await?;
        let data = resp.bytes().await?;
        Ok(Vec::from(data))
    }

    async fn get_texts(&self, product_id: u32, uuid: &str, page: u32) -> Result<TextPageData> {
        let dest = format!(
            "https://plus.pearson.com/eplayer/pdfassets/prod1/{product_id}/{uuid}/annotations/page{page}"
        );
        let resp = self.client.get(dest).send().await?;
        let text = resp.text().await?;
        Ok(sonic_rs::from_str::<Annotation>(&text)?.data)
    }
}

#[derive(Parser)]
struct Args {
    /// Copy and paste the value of the Cookie header.
    #[arg(short, long)]
    cookie: String,
    /// This is only necessary when you want to download links.
    /// Copy and paste the value of the X-Authorization header.
    #[arg(short, long)]
    auth_token: Option<String>,
    /// Copy and paste the product id of the book.
    #[arg(short, long)]
    product_id: u32,
    /// Copy and paste the uuid of the book.
    #[arg(short, long)]
    uuid: String,
    /// Output file path.
    #[clap(default_value = "out.pdf")]
    #[arg(short, long)]
    output_path: PathBuf,
}

#[tokio::main(flavor = "current_thread")]
async fn main() {
    let args = Args::parse();
    let extractor = Extractor::new(args.cookie, args.auth_token.unwrap_or_default()).unwrap();
    let output = File::create(args.output_path).unwrap();
    extractor
        .run(args.product_id, args.uuid, output)
        .await
        .unwrap();
}
