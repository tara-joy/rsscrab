
use std::fs::File;
use std::io::{BufRead, BufReader, Write};
use crate::error::RssGenError;

pub fn read_sites(file_path: &str) -> Result<Vec<String>, RssGenError> {
    let file = File::open(file_path).map_err(RssGenError::Io)?;
    let reader = BufReader::new(file);
    let mut sites = Vec::new();
    for line in reader.lines() {
        let line = line.map_err(RssGenError::Io)?;
        sites.push(line);
    }
    Ok(sites)
}

pub fn write_feeds(file_path: &str, feeds: &[String]) -> Result<(), RssGenError> {
    let mut file = File::create(file_path).map_err(RssGenError::Io)?;
    for feed in feeds {
        writeln!(file, "{}", feed).map_err(RssGenError::Io)?;
    }
    Ok(())
}
