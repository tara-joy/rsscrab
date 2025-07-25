mod io;
mod site_type;
mod rss_gen;
mod error;

use std::env;


fn print_usage() {
    println!("Usage:");
    println!("  rssgen --input <input_file> [--output <output_file>]");
    println!("  rssgen --url <site_url>");
    println!("If no output file is given, output will be written to rss-feeds.txt");
}

#[tokio::main]
async fn main() {
    let args: Vec<String> = env::args().collect();
    if args.len() == 1 || args.contains(&"--help".to_string()) {
        print_usage();
        return;
    }

    let mut input_file: Option<String> = None;
    let mut output_file: Option<String> = None;
    let mut input_url: Option<String> = None;
    let mut i = 1;
    while i < args.len() {
        match args[i].as_str() {
            "--input" => {
                if i + 1 < args.len() {
                    input_file = Some(args[i + 1].clone());
                    i += 1;
                }
            }
            "--output" => {
                if i + 1 < args.len() {
                    output_file = Some(args[i + 1].clone());
                    i += 1;
                }
            }
            "--url" => {
                if i + 1 < args.len() {
                    input_url = Some(args[i + 1].clone());
                    i += 1;
                }
            }
            _ => {}
        }
        i += 1;
    }

    if let Some(url) = input_url {
        let site = url.trim();
        let site_type = site_type::detect(site);
        match rss_gen::generate(site, &site_type).await {
            Ok(rss_url) => println!("{}", rss_url),
            Err(e) => eprintln!("Failed to generate RSS for: {} ({})", site, e),
        }
        return;
    }

    let input_file = match input_file {
        Some(f) => f,
        None => {
            print_usage();
            return;
        }
    };
    let output_file = output_file.unwrap_or_else(|| "rss-feeds.txt".to_string());

    let sites = match io::read_sites(&input_file) {
        Ok(s) => s,
        Err(e) => {
            eprintln!("Failed to read input file: {}", e);
            return;
        }
    };

    let mut rss_feeds = Vec::new();
    for site in sites {
        let site = site.trim();
        if site.is_empty() || site.starts_with('#') {
            continue;
        }
        let site_type = site_type::detect(site);
        match rss_gen::generate(site, &site_type).await {
            Ok(rss_url) => rss_feeds.push(rss_url),
            Err(e) => {
                eprintln!("Failed to generate RSS for: {} ({})", site, e);
                continue;
            }
        }
    }
    // Optionally, sort and deduplicate
    rss_feeds.sort();
    rss_feeds.dedup();
    if let Err(e) = io::write_feeds(&output_file, &rss_feeds) {
        eprintln!("Failed to write output file: {}", e);
    }
}
