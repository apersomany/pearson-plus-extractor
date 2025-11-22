# Pearson Plus Extractor

A Rust-based command-line tool to download Pearson Plus e-textbooks and convert them into PDF format. This tool fetches page images and text annotations to create a searchable PDF document.

## Prerequisites

- [Rust](https://www.rust-lang.org/tools/install) (Cargo)

## Installation

1. Clone this repository:
   ```bash
   git clone https://github.com/apersomany/pearson-plus-extractor.git
   cd pearson-plus-extractor
   ```

2. Build the project:
   ```bash
   cargo build --release
   ```

## Usage

Run the tool using `cargo run` or the built binary. You need to provide authentication details and book information extracted from your browser session.

```bash
cargo run --release -- \
  --cookie "YOUR_COOKIE_HEADER" \
  --product-id 123456 \
  --uuid "YOUR_BOOK_UUID" \
  --auth-token "YOUR_AUTH_TOKEN" \
  --output-path "my_textbook.pdf"
```

### Arguments

| Argument | Short | Description | Required |
|----------|-------|-------------|----------|
| `--cookie` | `-c` | The `Cookie` header value from your browser session. | Yes |
| `--product-id` | `-p` | The numeric product ID of the book. | Yes |
| `--uuid` | `-u` | The UUID of the book. | Yes |
| `--auth-token` | `-a` | The `X-Authorization` header value. | Optional* |
| `--output-path` | `-o` | Path for the output PDF file. Defaults to `out.pdf`. | No |

*Note: `auth-token` is marked as optional in the help text but is used in the request headers if provided. It is recommended to include it if available.*

## How to find the required values

1. Log in to Pearson Plus and open the e-textbook you want to download.
2. Open your browser's Developer Tools (F12 or Right-click -> Inspect).
3. Go to the **Network** tab.
4. Navigate through a few pages of the book to trigger network requests.
5. Look for a request that looks like `page1`, `page2`, or contains `pdfassets`.
   - The URL will typically look like: `https://plus.pearson.com/eplayer/pdfassets/prod1/{product_id}/{uuid}/pages/page{page_number}`.
6. From this URL, you can extract:
   - **Product ID**: The number after `prod1/`.
   - **UUID**: The string following the product ID.
7. Click on the request and look at the **Request Headers**:
   - Copy the entire value of the `Cookie` header.
   - Copy the value of the `X-Authorization` header.

## Disclaimer

This tool is for educational and personal archival purposes only. Please respect copyright laws and the terms of service of the content provider. Do not distribute copyrighted materials without permission.
