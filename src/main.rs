use clap::{App, Arg};
use dssim_core::{Dssim, DssimImage};
use image;
use imgref::ImgRef;
use std::fs;
use std::path::Path;
use std::process::Command;
use std::str;

fn make_dirs(skip_cache: bool) {
    let output_path = Path::new("output");
    let cache_path = Path::new("cache");

    if output_path.exists() {
        fs::remove_dir_all(output_path).expect("filed to clear output directory");
    }

    fs::create_dir(output_path).expect("failed to create output directory");

    if !skip_cache {
        if cache_path.exists() {
            fs::remove_dir_all(cache_path).expect("failed to clear cache directory");
        }

        fs::create_dir(cache_path).expect("failed to create cache directory");
    }
}

fn split_video(path: &Path, downscale: f32) {
    Command::new("ffmpeg")
        .arg("-i")
        .arg(path)
        .arg("-vf")
        .arg(format!("scale=iw*{ds:.2}:ih*{ds:.2}", ds = downscale))
        .arg("cache/%08d.png")
        .output()
        .expect("failed to execute ffmpeg");
}

fn open_to_dssim(dis: &Dssim, path: &Path) -> DssimImage<f32> {
    let img_rgb = image::open(path)
        .expect("failed to open image when trying to convert to dssim")
        .to_rgb();

    let img_rgb_norm = img_rgb
        .pixels()
        .map(|pix| Into::<rgb::RGB<f32>>::into(rgb::RGB::from(pix.0)) / 255.)
        .collect::<Vec<_>>();

    let img_ref = ImgRef::new(
        &img_rgb_norm,
        img_rgb.width() as usize,
        img_rgb.height() as usize,
    );

    dis.create_image(&img_ref)
        .expect("failed to create image with dssim")
}

fn image_seq_to_delta_vec(dis: &Dssim, dir: &Path, frames: i64) -> Vec<f64> {
    let mut delta_vec: Vec<f64> = vec![];
    let mut results_iter = fs::read_dir(dir).expect(&format!(
        "failed to read {} dir",
        &dir.file_name()
            .expect("failed to get directory name")
            .to_str()
            .expect("failed to convert directory name to str")
    ));

    let mut last = open_to_dssim(dis, &results_iter.next().unwrap().as_ref().unwrap().path());

    for result in results_iter {
        let current = open_to_dssim(dis, &result.as_ref().unwrap().path());
        delta_vec.push(dis.compare(&last, &current).0.into());
        last = current;

        println!(
            "{} / {}: {}",
            delta_vec.len(),
            frames,
            delta_vec.last().unwrap()
        );
    }

    delta_vec
}

fn take_subsect(path: &Path, range: (f64, f64), format: &str, img: bool) {
    println!("{:?}", range);

    let mut command = &mut Command::new("ffmpeg");

    command = command
        .arg("-ss")
        .arg(format!("{}s", range.0))
        .arg("-i")
        .arg(path);

    if img {
        command = command.arg("-vframes").arg("1");
    } else {
        command = command
            .arg("-t")
            .arg(format!("{}s", range.1))
            .arg("-vsync")
            .arg("0");
    }

    command
        .arg(format!(
            "./output/{}-{}.{}",
            range.0,
            range.0 + range.1,
            format
        ))
        .output()
        .expect("failed to slice with ffmpeg");
}

fn fetch_framerate(path: &Path) -> f64 {
    let output = Command::new("ffprobe")
        .arg("-v")
        .arg("error")
        .arg("-select_streams")
        .arg("v:0")
        .arg("-of")
        .arg("default=noprint_wrappers=1:nokey=1")
        .arg("-show_entries")
        .arg("stream=r_frame_rate")
        .arg(path)
        .output()
        .expect("failed to fecth framerate");

    if output.status.success() {
        let framerate_str = str::from_utf8(&output.stdout).unwrap().replace("\r\n", "");

        if framerate_str.contains("/") {
            let mut frac_iter = framerate_str.split("/");

            frac_iter.next().unwrap().parse::<f64>().unwrap()
                / frac_iter.next().unwrap().parse::<f64>().unwrap()
        } else {
            framerate_str.parse::<f64>().unwrap()
        }
    } else {
        panic!("ffprobe failed");
    }
}

fn fetch_frames(path: &Path) -> i64 {
    let mut output = Command::new("ffprobe")
        .arg("-v")
        .arg("error")
        .arg("-select_streams")
        .arg("v:0")
        .arg("-of")
        .arg("default=noprint_wrappers=1:nokey=1")
        .arg("-show_entries")
        .arg("stream=nb_frames")
        .arg(path)
        .output()
        .expect("failed to fecth frame count");

    let output_str = str::from_utf8(&output.stdout).unwrap();

    match output_str {
        "N/A\r\n" => {
            output = Command::new("ffprobe")
                .arg("-v")
                .arg("error")
                .arg("-count_frames")
                .arg("-select_streams")
                .arg("v:0")
                .arg("-of")
                .arg("default=noprint_wrappers=1:nokey=1")
                .arg("-show_entries")
                .arg("stream=nb_read_frames")
                .arg(path)
                .output()
                .expect("failed to fecth frame count");

            str::from_utf8(&output.stdout)
                .unwrap()
                .replace("\r\n", "")
                .parse::<i64>()
                .unwrap()
        }
        _ => output_str.replace("\r\n", "").parse::<i64>().unwrap(),
    }
}

fn calc_range(framerate: f64, start: i64, end: i64) -> (f64, f64) {
    let skip_start = start as f64 / framerate;

    (skip_start, end as f64 / framerate - skip_start)
}

fn main() {
    let matches = App::new("Split Scene")
        .version("0.1.0")
        .author("QuantumCoded <bfields32@student.cccs.edu>")
        .about("A tool to segmet a video by scene")
        .arg(
            Arg::with_name("downscale")
                .short("s")
                .value_name("DOWNSCALE")
                .help("The amount to scale the video down by"),
        )
        .arg(
            Arg::with_name("threshold")
                .short("t")
                .long("threshold")
                .value_name("THRESHOLD")
                .help("The dissimilarity threshold (smaller forces more dissimalar scenes)"),
        )
        .arg(
            Arg::with_name("format")
                .short("f")
                .long("format")
                .value_name("FORMAT")
                .help("The output file format"),
        )
        .arg(
            Arg::with_name("image")
                .short("i")
                .help("Export the first frame as an image"),
        )
        .arg(
            Arg::with_name("use_cache")
                .long("use-cache")
                .help("Uses an already eisting cache to lessen waiting"),
        )
        .arg(
            Arg::with_name("keep_cache")
                .long("keep-cache")
                .help("Skips the cleanup"),
        )
        .arg(
            Arg::with_name("INPUT")
                .help("Path to a video file")
                .required(true)
                .index(1),
        )
        .get_matches();

    let video_name = matches.value_of("INPUT").unwrap();
    let video_path = Path::new(video_name);

    let downscale = matches
        .value_of("downscale")
        .unwrap_or("0.1")
        .parse::<f32>()
        .unwrap();

    let threshold = matches
        .value_of("threshold")
        .unwrap_or("1")
        .parse::<f64>()
        .unwrap();

    let format = matches.value_of("format").unwrap_or("mkv");

    println!("fetching video framerate");
    let framerate = fetch_framerate(video_path);

    println!("fetching video frame count");
    let frames = fetch_frames(video_path);

    make_dirs(matches.is_present("use_cache"));

    if !matches.is_present("use_cache") {
        println!("splitting video frames");
        split_video(video_path, downscale);
    }

    let dis = Dssim::new();

    println!("loading images for comparison");
    let delta_vec = image_seq_to_delta_vec(&dis, Path::new("cache"), frames);

    let mut last_peak: i64 = -1;

    for (i, delta) in delta_vec.iter().enumerate() {
        if delta > &threshold {
            let range = calc_range(framerate, (last_peak + 1) as i64, i as i64);
            take_subsect(video_path, range, format, matches.is_present("image"));
            last_peak = i as i64;
        };
    }

    take_subsect(
        video_path,
        calc_range(framerate, (last_peak + 1) as i64, delta_vec.len() as i64),
        format,
        matches.is_present("image"),
    );

    if !matches.is_present("keep_cache") {
        fs::remove_dir_all(Path::new("cache")).expect("failed to clean up cache");
    }
}
