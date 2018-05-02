extern crate byteorder;

use zip;
use std::path::Path;
use std::fs::File;
use zip::ZipArchive;
use std::io::prelude::*;
use self::byteorder::{BigEndian, LittleEndian, ReadBytesExt};
use std::vec;

#[derive(Debug, Clone)]
pub struct FileData {
    rows: u64,
    cols: u64,
    points: Vec<f32>,
}

pub fn read_file(filename: &Path) -> FileData {
    let mut compressed = File::open(filename).expect("File won't open");
    let mut archive = ZipArchive::new(compressed).expect("Zip won't unzip");
    let mut cols: u64 = 0;
    let mut rows: u64 = 0;
    let mut cellsize: f64 = 0.0;
    let mut no_data: f32 = -9999.0;
    let mut floats = Vec::new();
    for i in 0..archive.len() {
        let mut file = archive.by_index(i).expect("Zipped file won't unzip");
        let name = file.sanitized_name();
        if let Some(ext) = name.extension() {
            if ext == "hdr" {
                // is header
                let mut data = String::new();
                file.read_to_string(&mut data);
                for line in data.lines() {
                    let mut components = line.split_whitespace();
                    match components.next() {
                        Some("nrows") => {
                            rows = components
                                .next()
                                .expect("Need to know rows")
                                .parse()
                                .expect("It needs to be a number");
                        }
                        Some("ncols") => {
                            cols = components
                                .next()
                                .expect("Need to know cols")
                                .parse()
                                .expect("It needs to be a number");
                        }
                        Some("cellsize") => {
                            cellsize = components.next().unwrap_or("0.0").parse().unwrap_or(0.0);
                        }
                        Some("NODATA_value") => {
                            no_data = components
                                .next()
                                .expect("Need to know the nodata value")
                                .parse()
                                .expect("Needs to be a number");
                        }
                        _ => {}
                    }
                }
            } else if ext == "flt" {
                // float
                // assume little-endian
                loop {
                    let f = file.read_f32::<LittleEndian>();
                    let f = match f {
                        Ok(a) => a,
                        _ => break,
                    };
                    floats.push(f);
                    println!("PUUUUUSH");
                    if f == no_data {
                        panic!("James said there's be none")
                    }
                }
            } else if ext == "prj" {
                // unhandled
            }
        }
    }

    FileData {
        rows: rows,
        cols: cols,
        points: floats,
    }
}
