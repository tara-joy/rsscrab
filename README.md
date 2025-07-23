# RSS Crab 🦀

RSS Crab is a simple tool to generate RSS feeds from blogs, video platforms, and more.

## Features

- Fetches and generates RSS feeds from different sites
- Easy to use, fast, and lightweight

## How to Use

1. **Build the app:**
   ```bash
   cargo build --release
   ```
2. **Run the app:**

   ```bash
   ./target/release/rsscrab <URL>
   ```

   Replace `<URL>` with the website you want to fetch a feed from.

3. **Get your RSS feed:**
   The app will print the RSS feed to your terminal or save it as needed.

## Requirements

- Rust (https://rust-lang.org)

## Example

```bash
./target/release/rsscrab https://example.com
```

---

Made with Rust 🦀
