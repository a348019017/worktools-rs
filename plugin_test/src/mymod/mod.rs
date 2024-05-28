use clap::builder::Str;
use clap::{Error, Parser};
use serde::de::value;
use serde_json::Number;
use std::fs::{self, create_dir_all};
use std::ops::Deref;
use std::path::{Path, PathBuf};
use std::time::{Duration, Instant};
//use error_chain::ChainedError;
use geo::algorithm::concave_hull;
use geo::algorithm::remove_repeated_points::RemoveRepeatedPoints;
use geo::{ConcaveHull, CoordsIter};
use geo::{ConvexHull, Coord, KNearestConcaveHull};
use geo_types::coord;
use geojson::{Feature, GeoJson, Geometry, Position, Value};
use glob::{glob_with, MatchOptions};
use itertools::Itertools;
use mycore::mycore::{BoundingBox, Vec3};
use rayon::{prelude::*, vec};
use serde::{Deserialize, Serialize};
mod mycore;
//use concave::concave_hull;
//use concave::point::Point;

/// Search for a pattern in a file and display the lines that contain it.
#[derive(Parser, Debug, Serialize, Deserialize, Clone)]
#[command(version, about, long_about = None)]
pub struct SingleArgs {
    /// 输出路径
    #[arg(short, long)]
    output: Option<std::path::PathBuf>,
    /// 输入路径，注意文件夹的结构，路径中需至少包含一个子文件夹用于分组
    #[arg(short, long)]
    input: Option<std::path::PathBuf>,
}

pub fn obj_single_parser(args: SingleArgs) -> Result<(), Box<dyn std::error::Error>> {
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

    let cur_path = _default_inputpath.join(".\\**\\*.obj");
    //
    //println!("当前文件夹有{:?}", cur_path);

    let files: Vec<_> = glob_with(cur_path.to_str().unwrap(), options)?
        .filter_map(|x: Result<std::path::PathBuf, glob::GlobError>| x.ok())
        .collect();

    //读取obj的全部顶点，通过
    if files.len() == 0 {
        println!("当前文件夹没有obj不能生成");
        return Ok(());
    }

    let result_boxes: Vec<_> = files
        .par_iter()
        .map(|path| {
            let (models, materials) = tobj::load_obj(path, &tobj::LoadOptions::default())
                .expect("Failed to OBJ load file");
            //同时计算高度和底面高度
            let mut i_heights: Vec<f32> = Vec::new();
            let mut i_xs: Vec<f32> = Vec::new();
            let mut i_ys: Vec<f32> = Vec::new();
            for (i, m) in models.iter().enumerate() {
                let mesh = &m.mesh;
                let vetex_num = mesh.positions.len() / 3;
                for j in 0..vetex_num {
                    let x = mesh.positions[j * 3] as f32;
                    let y = mesh.positions[j * 3 + 1] as f32;
                    let z = mesh.positions[j * 3 + 2] as f32;
                    i_heights.push(z);
                    i_xs.push(x);
                    i_ys.push(y);
                }
            }

            let _min = i_heights.iter().min_by(|a, b| a.total_cmp(b)).unwrap();
            let _max = i_heights.iter().max_by(|a, b| a.total_cmp(b)).unwrap();
            let _minx = i_xs.iter().min_by(|a, b| a.total_cmp(b)).unwrap();
            let _maxx = i_xs.iter().max_by(|a, b| a.total_cmp(b)).unwrap();
            let _miny = i_ys.iter().min_by(|a, b| a.total_cmp(b)).unwrap();
            let _maxy = i_ys.iter().max_by(|a, b| a.total_cmp(b)).unwrap();

            BoundingBox {
                min: Vec3::new(*_minx, *_miny, *_min),
                max: Vec3::new(*_maxx, *_maxy, *_max),
            }
        })
        .collect();

    let mut firstBbox = result_boxes[0];

    //println!("1初始计算的bbox{:?}", firstBbox.center());

    for item in 1..result_boxes.len() {
        let last_bbox = &result_boxes[item];
        //println!("2初始计算的bbox{:?}", last_bbox.center());
        let newfirst_bbox = firstBbox.union(last_bbox);
        firstBbox = newfirst_bbox;
    }

    let center_vec = firstBbox.center();

    // println!(
    //     "初始计算的bbox{};{};{}",
    //     center_vec.x, center_vec.y, center_vec.z
    // );

    //这里有些麻烦，首先需要联合计算obj的box，然后unionbox作为中心点，最后返回合并的，最后逐个的计算导出
    let result: Vec<_> = files
        .par_iter()
        .map(|path| {
            let filename = path.file_stem().unwrap().to_str().unwrap();
            let (models, materials) = tobj::load_obj(path, &tobj::LoadOptions::default())
                .expect("Failed to OBJ load file");

            //同时计算高度和底面高度
            let mut icoords: Vec<Coord> = Vec::new();
            let mut icoordpts: Vec<geo::Point> = Vec::new();
            //let mut icoords2: Vec<Point> = Vec::new();
            let mut i_heights: Vec<f64> = Vec::new();
            let mut i_xs: Vec<f64> = Vec::new();
            let mut i_ys: Vec<f64> = Vec::new();
            for (i, m) in models.iter().enumerate() {
                let mesh = &m.mesh;
                let vetex_num = mesh.positions.len() / 3;

                for j in 0..vetex_num {
                    let x = mesh.positions[j * 3] as f64;
                    let y = mesh.positions[j * 3 + 1] as f64;
                    let z = mesh.positions[j * 3 + 2] as f64;
                    //icoords.push(coord! {x:x,y:y});
                    i_heights.push(z);
                    i_xs.push(x);
                    i_ys.push(y);
                }
            }

            let _min = i_heights.iter().min_by(|a, b| a.total_cmp(b)).unwrap();
            let _max = i_heights.iter().max_by(|a, b| a.total_cmp(b)).unwrap();
            let _height = _max - _min;

            let mut properties = geojson::JsonObject::new();

            properties.insert("minHeight".to_string(), geojson::JsonValue::from(*_min));
            properties.insert("maxHeight".to_string(), geojson::JsonValue::from(*_max));
            properties.insert("height".to_string(), geojson::JsonValue::from(_height));
            properties.insert("name".to_string(), geojson::JsonValue::from(filename));

            for j in 0..i_heights.len() {
                let x = i_xs[j] - center_vec.x as f64;
                let y = i_ys[j] - center_vec.y as f64;
                //let z = mesh.positions[j * 3 + 2] as f64;
                icoordpts.push(coord! {x:x,y:y}.into());
                //icoords2.push(Point::new(x, y, j as u64));
            }
            //let _ = icoordpts.into_iter().unique();
            let mps = geo::MultiPoint(icoordpts).remove_repeated_points();

            // //icoords2.into_iter().u
            // let concave_polygon2 = concave_hull(&mut icoords2, 10, false);
            // println!("testpositions{:?}", concave_polygon2);
            // let concave_polygon: Vec<Position> =
            //     concave_polygon2.iter().map(|f| vec![f.x, f.y]).collect();

            //println!("{:?}", concave_polygon2);
            //次算法效果还不是太好哦
            let concave_polygon: geo::Polygon = mps.convex_hull();
            // let _rst = geojson::Feature {
            //     bbox: None,
            //     //geometry: Some(Geometry::new(Value::Polygon(vec![concave_polygon]))),
            //     geometry: Some(Geometry::from(&concave_polygon)),
            //     id: None,
            //     // See the next section about Feature properties
            //     properties: Some(properties),
            //     foreign_members: None,
            // };
            let _rst = geojson::Feature {
                bbox: None,
                //geometry: Some(Geometry::new(Value::Polygon(vec![concave_polygon]))),
                geometry: Some(Geometry::from(&concave_polygon)),
                id: None,
                // See the next section about Feature properties
                properties: Some(properties),
                foreign_members: None,
            };
            _rst
            //使用georust的concavehull方法计算，并导出为geojson进行测试
        })
        .collect();

    let _ = fs::write(
        "./test,geojson",
        geojson::FeatureCollection::from_iter(result).to_string(),
    );

    Ok(())
}
