extern crate bincode;
extern crate byteorder;


use self::byteorder::{LittleEndian, ReadBytesExt};
use std::fs;
use std::fs::File;
use std::io::prelude::*;
use std::path::Path;
use zip::ZipArchive;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FileData {
    pub rows: u64,
    pub cols: u64,
    pub points: Vec<f32>,
    pub min_height: f32,
    pub max_height: f32,
    pub cellsize: f64,
    pub x_lower_left_corner: f64,
    pub y_lower_left_corner: f64,
}

pub fn read_file(filename: &Path, compressed: bool) -> FileData {
    match compressed{
        true => read_file_compressed(filename),
        false => read_file_uncompressed(filename),
    }
}

fn read_file_compressed(filename: &Path) -> FileData {
    let mut archive;
    let compressed = File::open(filename).expect("File won't open");
    archive = ZipArchive::new(compressed).expect("Zip won't unzip");
    let mut cols: u64 = 0;
    let mut rows: u64 = 0;
    let mut cellsize: f64 = 0.0;
    let mut xllcorner: f64 = 0.0;
    let mut yllcorner: f64 = 0.0;
    let mut no_data: f32 = -9999.0;
    let mut max_height: f32 = -::std::f32::INFINITY;
    let mut min_height: f32 = ::std::f32::INFINITY;
    let mut floats = Vec::new();
    for i in 0..archive.len() {
        let mut file = archive.by_index(i).expect("Zipped file won't unzip");
        let name = file.sanitized_name();
        if let Some(ext) = name.extension() {
            if ext == "hdr" {
                // is header
                let mut data = String::new();
                file.read_to_string(&mut data).expect("Can't read into string");
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
                        Some("xllcorner") => {
                            xllcorner = components.next().unwrap_or("0.0").parse().unwrap_or(0.0);
                        }
                        Some("yllcorner") => {
                            yllcorner = components.next().unwrap_or("0.0").parse().unwrap_or(0.0);
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
                    if min_height > f {
                        min_height = f;
                    }
                    if max_height < f {
                        max_height = f;
                    }
                    floats.push(f);
                    if f == no_data {
                        panic!("James said there's be none")
                    }
                }
            } else if ext == "prj" {
                // unhandled
            }
        }
    }
    let data = FileData {
        rows,
        cols,
        points: floats,
        min_height,
        max_height,
        cellsize,
        x_lower_left_corner: xllcorner,
        y_lower_left_corner: yllcorner,
    };
    //let mut dest = File::create(cache_path).expect("Must be path");
    //bincode::serialize_into(dest, &data);
    data
}

// doesn't appear to work for now
fn read_file_uncompressed(folder: &Path) -> FileData {
    let folder = fs::read_dir(folder).expect("Folder won't open");
    let mut cols: u64 = 0;
    let mut rows: u64 = 0;
    let mut cellsize: f64 = 0.0;
    let mut xllcorner: f64 = 0.0;
    let mut yllcorner: f64 = 0.0;
    let mut no_data: f32 = -9999.0;
    let mut max_height: f32 = -::std::f32::INFINITY;
    let mut min_height: f32 = ::std::f32::INFINITY;
    let mut floats = Vec::new();
    for path_maybe in folder {
        if let Ok(dir_entry) = path_maybe {
            let path = dir_entry.path();
            println!("Path: {:?}", path);
            let mut file = File::open(path.clone()).expect("File won't open");
            if let Some(ext) = path.extension() {
                if ext == "hdr" {
                    // is header
                    let mut data = String::new();
                    file.read_to_string(&mut data).expect("Can't read into string");
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
                            Some("xllcorner") => {
                                xllcorner = components.next().unwrap_or("0.0").parse().unwrap_or(0.0);
                            }
                            Some("yllcorner") => {
                                yllcorner = components.next().unwrap_or("0.0").parse().unwrap_or(0.0);
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
                        if min_height > f {
                            min_height = f;
                        }
                        if max_height < f {
                            max_height = f;
                        }
                        floats.push(f);
                        if f == no_data {
                            panic!("James said there's be none")
                        }
                    }
                } else if ext == "prj" {
                    // unhandled
                }
            }
        }
    }
    FileData {
        rows,
        cols,
        points: floats,
        min_height,
        max_height,
        cellsize,
        x_lower_left_corner: xllcorner,
        y_lower_left_corner: yllcorner,
    }
}