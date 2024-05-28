use clap::builder::Str;
use clap::{Error, Parser};
use image::imageops::thumbnail;
use image::{io::Reader as ImageReader, GenericImage, GenericImageView, ImageError};
use image::{open, RgbImage};
use serde_json::Number;
use std::fs::{self, create_dir_all};
use std::path::{Path, PathBuf};
use std::time::{Duration, Instant};
//use error_chain::ChainedError;
use glob::{glob_with, MatchOptions};
use rayon::prelude::*;
use serde::{Deserialize, Serialize};
//use serde_json::Result;

// #[derive(Serialize, Deserialize)]
// pub struct PIndex {
//     /// 相比json模块，我们可以少些很多数据类型不同代理的问题。
//     name: String,
//     title: String,
//     body: String,
// }

const MIN_SIZE: u32 = 5000;
const THUMBNAIL_WIDTH: u32 = 1024;
const THUMBNAIL_HEIGHT: u32 = 512;

#[derive(Serialize, Deserialize, Debug)]
pub struct PGroup {
    name: String,
    images: Vec<PImage>,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct PImage {
    imagename: String,
    lonlat: Option<Vec<f64>>,
    height: Option<f64>,
    longitudeoffset: Option<f64>,
    usetile: bool,
}

/// Search for a pattern in a file and display the lines that contain it.
#[derive(Parser, Debug, Serialize, Deserialize)]
#[command(version, about, long_about = None)]
struct Cli {
    /// 输出路径
    #[arg(short, long)]
    output: Option<std::path::PathBuf>,
    /// 输入路径，注意文件夹的结构，路径中需至少包含一个子文件夹用于分组
    #[arg(short, long)]
    input: Option<std::path::PathBuf>,
}

//测试多线程导出操作，还多线程个毛线，debug是release的n倍
fn main() -> Result<(), Box<dyn std::error::Error>> {
    let _ = clip_image_entry();
    Ok(())
}

pub fn clip_image_entry() -> Result<(), Box<dyn std::error::Error>> {
    let args = Cli::parse();
    let mut options: MatchOptions = Default::default();
    options.case_sensitive = false;

    let mut _default_inputpath = PathBuf::new();
    if let Some(input) = args.input {
        _default_inputpath = input;
    }
    let mut _default_outputpath = PathBuf::new();
    if let Some(output) = args.output {
        _default_outputpath = output;
    }
    let mut groups = Vec::new();
    for entry in fs::read_dir(_default_inputpath)? {
        let entry = entry?;
        let path = entry.path();
        if path.is_dir() {
            let group = clip_image_tiles(&path, &_default_outputpath).unwrap();
            groups.push(group);
        }
    }
    let output_json = serde_json::to_string(&groups)?;
    let output_json_path = _default_outputpath.join("qindex.json");
    let _ = fs::write(output_json_path, output_json);
    println!("全景切片导出完成！");
    Ok(())
}

pub fn excute(options: &str) -> Result<(), Box<dyn std::error::Error>> {
    let args: Cli = serde_json::from_str(options)?;
    let mut options: MatchOptions = Default::default();
    options.case_sensitive = false;

    let mut _default_inputpath = PathBuf::new();
    if let Some(input) = args.input {
        _default_inputpath = input;
    }
    let mut _default_outputpath = PathBuf::new();
    if let Some(output) = args.output {
        _default_outputpath = output;
    }
    let mut groups = Vec::new();
    for entry in fs::read_dir(_default_inputpath)? {
        let entry = entry?;
        let path = entry.path();
        if path.is_dir() {
            let group = clip_image_tiles(&path, &_default_outputpath).unwrap();
            groups.push(group);
        }
    }
    let output_json = serde_json::to_string(&groups)?;
    let output_json_path = _default_outputpath.join("qindex.json");
    let _ = fs::write(output_json_path, output_json);
    println!("全景切片导出完成！");
    Ok(())
}

//裁切多张图片，保留原始图片的结构
fn clip_image_tiles(input: &Path, output: &Path) -> Result<PGroup, Box<dyn std::error::Error>> {
    //获取文件夹名称
    let filename = input.file_name().unwrap().to_str().unwrap();
    let cur_path = input.join("./*.jpg");
    let mut options: MatchOptions = Default::default();
    options.case_sensitive = false;
    let files: Vec<_> = glob_with(cur_path.to_str().unwrap(), options)?
        .filter_map(|x: Result<std::path::PathBuf, glob::GlobError>| x.ok())
        .collect();
    if files.len() == 0 {
        println!("文件夹无全景图")
    }
    //文件名创建
    let newgroupfolder = output.join(filename);
    create_dir_all(&newgroupfolder)?;

    let mut _image_group = PGroup {
        name: filename.to_string(),
        images: Vec::new(),
    };
    let image_result: Vec<_> = files
        .par_iter()
        .map(|path| {
            clip_image_tile(path, &newgroupfolder)
                .map_err(|e| path.display().to_string())
                .unwrap()
        })
        .collect();
    _image_group.images = image_result;
    Ok(_image_group)
}

fn clip_image_tile(input: &Path, output: &Path) -> Result<PImage, Box<dyn std::error::Error>> {
    let org_img = open(input)?;
    let img = org_img.into_rgb8();
    let filename = input.file_stem().unwrap().to_str().unwrap();
    //if args.input
    let newfolder = output.join(filename);
    let clone_new_folder = newfolder.to_str().unwrap();
    fs::create_dir_all(&newfolder)?; //默认创建目录
    let _width = img.width();
    let _height = img.height();
    let _rownum = 4; //默认8行4列
    let _colnum = 8;
    //按照2:1确定最终尺寸，要求宽度是8的倍速，resize性能未知,补齐到8倍数
    //let _ratio=_width/_height;操作速度还是太慢了
    let iswidthlong = _width >= 2 * _height;
    let is_max = _width >= MIN_SIZE;

    //默认创建缩略图
    let thm_outputf = newfolder.join(format!("{}_low.JPG", filename));
    println!("导出缩略图{:?}", thm_outputf);
    let _ = open(input)?
        .resize(THUMBNAIL_WIDTH, THUMBNAIL_HEIGHT, image::imageops::Nearest)
        .save(thm_outputf);

    if iswidthlong && is_max {
        let _nwidth = _width;
        let _nheight = _width / 2;
        let _offset_height = (_nheight - _height) / 2;
        let mut imgbuf = RgbImage::new(_nwidth, _nheight);
        imgbuf
            .sub_image(0, _offset_height, _width, _height)
            .copy_from(&img, 0, 0)?;
        let tilesize = _nwidth / 8; //此算法会导致缺少8个像素数据的情况
                                    // for i in 0.._rownum {
                                    //     for j in 0.._colnum {
                                    //         let region = imgbuf.view (
                                    //             j*tilesize,i*tilesize,tilesize,tilesize
                                    //         );
                                    //         let  regionimgbuf =region.to_image();
                                    //         let newfilename = format!("{}/row-{}-column-{}.jpg", filename, i + 1, j + 1);
                                    //         println!("{}文件夹导出{}",&newfilename,tilesize);
                                    //         let _=regionimgbuf.save(newfilename);
                                    //     }
                                    // }
        (0.._rownum * _colnum).into_par_iter().for_each(|x| {
            let i = x / 8;
            let j = x % 8;

            let region = imgbuf.view(j * tilesize, i * tilesize, tilesize, tilesize);
            let regionimgbuf = region.to_image();
            let newfilename = PathBuf::new().join(clone_new_folder).join(format!(
                "row-{}-column-{}.jpg",
                i + 1,
                j + 1
            ));
            println!("{}文件导出", newfilename.to_str().unwrap());
            let _ = regionimgbuf.save(newfilename);
        });
        //imgbuf.save("test.jpg")?;
        //img.resize(_nwidth, _nheight,image::imageops::FilterType::Nearest).save("test.jpg")?;
    } else {
        println!("全景图{}不符合要求，暂未处理", filename)
        //Ok(())
    }
    //
    let mut _image_info = PImage {
        imagename: filename.to_string(),
        height: None,
        lonlat: None,
        longitudeoffset: None,
        usetile: true,
    };
    let mut _lonlat = vec![0.0f64, 0.0f64, 0.0f64];

    //解析exif的相关信息
    let file = std::fs::File::open(input)?;
    let mut bufreader = std::io::BufReader::new(&file);
    let exifreader = exif::Reader::new();
    let exif = exifreader.read_from_container(&mut bufreader)?;

    //AbsoluteAltitude
    //GPSAltitude
    //GPSLatitude
    match exif.get_field(exif::Tag::GPSAltitude, exif::In::PRIMARY) {
        Some(xres) => match xres.value {
            exif::Value::Rational(ref v) if !v.is_empty() => {
                //println!("GPSAltitude {}", v[0].to_f64())
                _lonlat[2] = v[0].to_f64();
            }
            _ => eprintln!("GPSAltitude value is broken"),
        },
        None => eprintln!("GPSAltitude tag is missing"),
    }

    match exif.get_field(exif::Tag::GPSLatitude, exif::In::PRIMARY) {
        Some(xres) => match xres.value {
            exif::Value::Rational(ref v) if !v.is_empty() => {
                // println!(
                //     "GPSLatitude {},{},{}",
                //     v[0].to_f64(),
                //     v[1].to_f64(),
                //     v[2].to_f64()
                // )
                _lonlat[1] = v[0].to_f64() + v[1].to_f64() / 60.0f64 + v[2].to_f64() / 3600.0f64;
            }
            _ => eprintln!("GPSLatitude value is broken"),
        },
        None => eprintln!("GPSLatitude tag is missing"),
    }

    match exif.get_field(exif::Tag::GPSLongitude, exif::In::PRIMARY) {
        Some(xres) => match xres.value {
            exif::Value::Rational(ref v) if !v.is_empty() => {
                // println!(
                //     "GPSLongitude {},{},{}",
                //     v[0].to_f64(),
                //     v[1].to_f64(),
                //     v[2].to_f64()
                // )
                _lonlat[0] = v[0].to_f64() + v[1].to_f64() / 60.0f64 + v[2].to_f64() / 3600.0f64;
            }
            _ => eprintln!("GPSLongitude value is broken"),
        },
        None => eprintln!("GPSLongitude tag is missing"),
    }
    _image_info.lonlat = Option::Some(_lonlat);
    // for f in exif.fields() {
    //     println!(
    //         "{} {} {}",
    //         f.tag,
    //         f.ifd_num,
    //         f.display_value().with_unit(&exif)
    //     );
    // }
    Ok(_image_info)
}
