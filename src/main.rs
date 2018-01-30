extern crate las;
extern crate kdtree;

use std::env;
use las::{Reader, Writer};
use kdtree::KdTree;
use kdtree::ErrorKind;
use kdtree::distance::squared_euclidean;

fn print_infos(header: &las::header::Header) {
    let (major, minor) = header.version;
    println!("Las version: {}.{}", major, minor);
    println!("{}", header.point_format);
    println!("\tBytes per points: {}", header.point_format.len());
    println!("\tGPS? {}", header.point_format.has_gps_time());
    println!("\tRGB? {}", header.point_format.has_color());

    let bbox = header.bounds;
    let bbox_dim = vec![
        bbox.max.x - bbox.min.x,
        bbox.max.y - bbox.min.y,
        bbox.max.z - bbox.min.z
    ];
    println!("Number of points: {}", header.number_of_points);
    println!("Bounding Box min:  [x: {}, y: {}, z: {}]", bbox.min.x, bbox.min.y, bbox.min.z);
    println!("Bounding Box max:  [x: {}, y: {}, z: {}]", bbox.max.x, bbox.max.y, bbox.max.z);
    println!("Bounding Box size: [x: {}, y: {}, z: {}]", bbox_dim[0], bbox_dim[1], bbox_dim[2]);
}


fn copy(filename: &String, new_las_format: (u8, u8), new_point_format: u8) {
    let mut reader = Reader::from_path(filename).unwrap();

    let old_header = reader.header.clone();
    let mut new_header = reader.header.clone();
    new_header.point_format = new_point_format.into();
    new_header.version = new_las_format;


    let old_format = &old_header.point_format;
    let new_format = new_header.point_format.clone();

    let mut writer = Writer::from_path("output.las", new_header).unwrap();
    let points = reader.points();
    for point in points {
        let mut p = point.unwrap();

        if !old_format.has_gps_time() && new_format.has_gps_time() {
            p.gps_time = Some(Default::default());
        } else if old_format.has_gps_time() && !new_format.has_gps_time() {
            p.gps_time = None
        }

        if !old_format.has_color() && new_format.has_color() {
            p.color = Some(las::point::Color{red: 0, green: 0, blue: 0});
        } else if old_format.has_color() && !new_format.has_color() {
            p.color = None
        }

        writer.write(&p).unwrap();
    }
}


fn merge_rgb(rgb_filename: &String, no_rgb_filename: &String) {
    const  dimensions : usize = 3;
    let mut tree = KdTree::new(dimensions);

    let mut reader = Reader::from_path(rgb_filename).unwrap();

    let mut points_w_rgb = Vec::<las::point::Point>::with_capacity(reader.header.number_of_points as usize);
    let points_iter = reader.points();

    println!("Reading & constructing tree");
    for (i, point) in points_iter.enumerate() {
        let p = point.unwrap();
        tree.add([p.x, p.y, p.z], p);
        points_w_rgb.push(p);
    }

    let mut reader_2 = Reader::from_path(no_rgb_filename).unwrap();
    let mut points_wo_rgb = Vec::<las::point::Point>::with_capacity(reader_2.header.number_of_points as usize);

    let mut p: [f64; dimensions];
    println!("Querying");
    for point_read in reader_2.points() {
        let mut point = point_read.unwrap();
        p = [point.x, point.y, point.z];
        let res = tree.nearest(&p, 1, &squared_euclidean).unwrap();
        let (_dist, nearest_point) = res[0];
        point.intensity = nearest_point.intensity;
        points_wo_rgb.push(point);
    }
    println!("Ok")

}

fn main() {
    let args: Vec<String> = env::args().collect();
    let input_file = &args[1];

    let reader = Reader::from_path(input_file).unwrap();
    let header = reader.header;
    print_infos(&header);
//    copy(&input_file, (1,2), 1);
    merge_rgb(&args[1], &args[2]);

}
