use chrono::DateTime;
use serde::{Deserialize, Serialize};
use serde_xml_rs::from_str;
use std::fs;

fn get_health_files() -> Option<impl Iterator<Item = String>> {
    let current_file_path = fs::canonicalize(file!()).ok()?;

    let workouts_dir = current_file_path
        .parent()?
        .parent()?
        .join("apple_health_export/workout-routes");

    let files = fs::read_dir(workouts_dir)
        .ok()?
        .flat_map(|entry| entry)
        .map(|entry| entry.path())
        .map(|path| fs::read_to_string(path).unwrap());

    Some(files)
}

#[derive(Debug, Serialize, Deserialize, PartialEq)]
struct Gpx {
    trk: Trk,
    metadata: Metadata,
}

#[derive(Debug, Serialize, Deserialize, PartialEq)]
struct Metadata {
    time: String,
}

#[derive(Debug, Serialize, Deserialize, PartialEq)]
struct Trk {
    name: String,
    trkseg: Trkseg,
}

#[derive(Debug, Serialize, Deserialize, PartialEq)]
struct Trkseg {
    trkpt: Vec<Trkpt>,
}

#[derive(Debug, Serialize, Deserialize, PartialEq)]
struct Trkpt {
    lon: f64,
    lat: f64,
    ele: f64,
    time: String,
    extensions: Extensions,
}

#[derive(Debug, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "camelCase")]
struct Extensions {
    speed: f64,
    course: f64,
    h_acc: f64,
    v_acc: f64,
}

fn main() {
    let mut gpxs: Vec<Gpx> = get_health_files()
        .unwrap()
        .map(|file| from_str(&file).unwrap())
        .collect();

    gpxs.sort_by_cached_key(|gpx| {
        DateTime::parse_from_rfc3339(&gpx.trk.trkseg.trkpt[0].time).unwrap()
    });

    for gpx in gpxs {
        let mut average_speed = 0.0f64;
        let mut top_speed = 0.0f64;
        let mut count = 0;

        for trkpt in gpx.trk.trkseg.trkpt {
            average_speed += trkpt.extensions.speed;
            top_speed = top_speed.max(trkpt.extensions.speed);
            count += 1;
        }

        average_speed /= count as f64;

        average_speed *= 3.6; // m/s -> km/h
        top_speed *= 3.6; // m/s -> km/h

        println!("{}", gpx.trk.name);
        println!("Average speed: {} km/h", average_speed);
        println!("Top speed: {} km/h", top_speed);
        println!();
    }
}
