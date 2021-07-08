#[macro_use]
extern crate clap;

use clap::Arg;

fn main() {
    let matches = app_from_crate!()
        .arg(
            Arg::with_name("INPUT")
                .index(1)
                .required(true)
                .help("Path to video file"),
        )
        .arg(
            Arg::with_name("scale")
                .short("s")
                .long("scale")
                .value_name("SCALE")
                .default_value("0.1")
                .help("The scale factor to apply before splitting"),
        )
        .arg(
            Arg::with_name("threshold")
                .long("threshold")
                .short("t")
                .value_name("THRESHOLD")
                .default_value("1")
                .help("The threshold to use when splitting"),
        )
        .arg(
            Arg::with_name("lookahead")
                .long("lookahead")
                .short("l")
                .value_name("FRAMES")
                .default_value("1")
                .help("The number of frames to compare against"),
        )
        .arg(
            Arg::with_name("format")
                .short("f")
                .long("format")
                .value_name("FORMAT")
                .default_value("mkv")
                .possible_values(&["mkv", "mp4", "png"])
                .help("The output file format"),
        )
        .arg(
            Arg::with_name("slideshow")
                .long("slideshow")
                .help("Export images instead of videos, used for splitting slideshows"),
        )
        .arg(
            Arg::with_name("keepcache")
                .long("keep-cache")
                .help("Leave the frames cache behind after splitting"),
        )
        .get_matches();

    let _input = matches.value_of("INPUT").unwrap();
    let _scale: f32 = matches
        .value_of("scale")
        .unwrap()
        .parse()
        .expect("could not parse scale value");
    let _threshold: f64 = matches
        .value_of("threshold")
        .unwrap()
        .parse()
        .expect("could not parse threshold value");
    let _lookahead: u8 = matches
        .value_of("lookahead")
        .unwrap()
        .parse()
        .expect("could not parse lookahead value");
    let _format = matches.value_of("format").unwrap();
    let _slideshow = matches.is_present("slideshow");
    let _keep_cache = matches.is_present("keepcache");
}
