use crate::App;
use dssim_core::{Dssim, DssimImage};
use imgref::ImgRef;
use rgb::RGB;
use std::{fs, io, ops::Range, ops::RangeInclusive, path::PathBuf, process::Command};
use tui::{backend::Backend, Terminal};

pub fn get_framerate(term: &mut Terminal<impl Backend>, app: &mut App) -> io::Result<()> {
    app.info("get framerate");
    term.draw(|f| app.draw(f))?;

    let output = Command::new("ffprobe")
        .args([
            "-v",
            "error",
            "-select_streams",
            "v:0",
            "-of",
            "default=noprint_wrappers=1:nokey=1",
            "-show_entries",
            "stream=r_frame_rate",
        ])
        .arg(app.input())
        .output()
        .expect("failed to get framerate");

    if output.status.success() {
        let fr = std::str::from_utf8(&output.stdout)
            .expect("failed to parse ffprobe output")
            .trim();

        let fr = if fr.contains("/") {
            let mut frac = fr.split("/");
            let a: f64 = frac
                .next()
                .unwrap()
                .parse()
                .expect("failed to parse framerate fraction");
            let b: f64 = frac
                .next()
                .unwrap()
                .parse()
                .expect("failed to parse framerate fraction");

            a / b
        } else {
            fr.parse().expect("failed to parse ffprobe output")
        };

        app.set_framerate(fr);
        app.info(format!("framerate is {} fps", fr));
        term.draw(|f| app.draw(f))?;
    } else {
        if app.input().exists() {
            panic!("ffprobe failed");
        } else {
            panic!("input file does not exist");
        }
    }

    Ok(())
}

pub fn get_frame_count(term: &mut Terminal<impl Backend>, app: &mut App) -> io::Result<()> {
    app.info("get frame count");
    term.draw(|f| app.draw(f))?;

    let output = Command::new("ffprobe")
        .args([
            "-v",
            "error",
            "-select_streams",
            "v:0",
            "-of",
            "default=noprint_wrappers=1:nokey=1",
            "-show_entries",
            "stream=nb_frames",
        ])
        .arg(app.input())
        .output()
        .expect("failed to get frame count");
    let output = std::str::from_utf8(&output.stdout).unwrap().trim();

    let fc: usize = match output {
        "N/A" => {
            let output = String::from_utf8(
                Command::new("ffprobe")
                    .args([
                        "-v",
                        "error",
                        "-count_frames",
                        "-select_streams",
                        "v:0",
                        "-of",
                        "default=noprint_wrappers=1:nokey=1",
                        "-show_entries",
                        "stream=nb_read_frames",
                    ])
                    .arg(app.input())
                    .output()
                    .expect("failed to get frame")
                    .stdout,
            )
            .unwrap();
            let output = output.trim();

            output
                .parse()
                .expect("could not parse frame count from ffprobe")
        }

        _ => output
            .parse()
            .expect("could not parse frame count from ffprobe"),
    };

    app.set_frame_count(fc);
    app.info(format!("got {} frames", fc));
    term.draw(|f| app.draw(f))?;

    Ok(())
}

pub fn create_image_sequence(
    term: &mut Terminal<impl Backend>,
    app: &mut App,
    scale: f64,
) -> io::Result<()> {
    app.info("create image sequence");
    app.info("make cache");
    term.draw(|f| app.draw(f))?;

    let path = app.input().parent().unwrap().join(format!(
        "frames_{}",
        app.input().file_name().unwrap().to_str().unwrap()
    ));

    if path.exists() {
        app.info("found existing cache");
        term.draw(|f| app.draw(f))?;

        app.set_cache(&path);
        app.info(format!(
            "cache set to '{}'",
            path.file_name().unwrap().to_str().unwrap()
        ));
        term.draw(|f| app.draw(f))?;
    } else {
        fs::create_dir(&path)?;
        app.info("created cache");
        term.draw(|f| app.draw(f))?;

        app.set_cache(&path);
        app.info(format!(
            "cache set to '{}'",
            path.file_name().unwrap().to_str().unwrap()
        ));
        term.draw(|f| app.draw(f))?;

        app.info("splitting...");
        term.draw(|f| app.draw(f))?;

        Command::new("ffmpeg")
            .arg("-i")
            .arg(app.input())
            .arg("-vf")
            .arg(format!("scale=iw*{s:.2}:ih*{s:.2}", s = scale))
            .arg(app.cache().unwrap().join("%00d.png"))
            .output()
            .expect("failed to split to image sequence");

        app.info("created image sequence");
        term.draw(|f| app.draw(f))?;
    }

    Ok(())
}

pub fn compare_frames(
    term: &mut Terminal<impl Backend>,
    app: &mut App,
    threshold: f64,
    lookahead: u8,
) -> io::Result<Vec<RangeInclusive<usize>>> {
    app.info("compare frames");
    term.draw(|f| app.draw(f))?;

    let dssim = Dssim::new();
    let frames: Vec<PathBuf> = fs::read_dir(app.cache().unwrap())
        .unwrap()
        .map(|result| result.expect("failed to read image from cache").path())
        .collect();
    let windows = frames.windows(1 + lookahead as usize);
    let mut scenes = Vec::new();

    let mut last_windows = Vec::new();
    let mut last_scene = 0;

    for (idx, window) in windows.enumerate() {
        let window: Vec<DssimImage<f32>> = window
            .iter()
            .map(|path| {
                let img = image::open(path)
                    .expect("failed to open image from cache")
                    .to_rgb8();
                let buf: Vec<RGB<f32>> = img
                    .pixels()
                    .map(|pix| Into::<RGB<f32>>::into(RGB::from(pix.0)) / u8::MAX as f32)
                    .collect();
                let img_ref = ImgRef::new(&buf, img.width() as usize, img.height() as usize);

                dssim
                    .create_image(&img_ref)
                    .expect("failed to create dssim image")
            })
            .collect();

        let img = window.first().unwrap();
        let diff: Vec<f64> = window[1..]
            .iter()
            .map(|alt| dssim.compare(&img, alt).0.into())
            .collect();
        let min = diff.iter().fold(f64::INFINITY, |a, &b| a.min(b));

        if min > threshold {
            scenes.push(last_scene..=idx);
            // we got a scene change
            last_scene = idx
        }

        app.progress_compare(threshold, min);
        app.info(format!(
            "compare frame {} to {} [lookahead = {}]: {} ",
            app.get_progress(),
            app.get_progress() + 1,
            lookahead,
            min
        ));
        term.draw(|f| app.draw(f))?;

        last_windows = window;
    }

    // for in whatevers left get slices and then get the first and compare to the rest
    let mut frames: Vec<&DssimImage<f32>> = last_windows.iter().skip(1).collect();

    while frames.len() > 1 {
        let img = frames.first().unwrap();
        let diff: Vec<f64> = frames[1..]
            .iter()
            .map(|alt| dssim.compare(&img, *alt).0.into())
            .collect();
        let min = diff.iter().fold(f64::INFINITY, |a, &b| a.min(b));

        if min > threshold {
            // we got a scene change
            // ignoring this for rn
        }

        app.progress_compare(threshold, min);
        app.info(format!(
            "compare frame {} to {} [lookahead = {}]: {} ",
            app.get_progress(),
            app.get_progress() + 1,
            lookahead,
            min
        ));
        term.draw(|f| app.draw(f))?;

        frames = frames[1..].to_vec();
    }

    println!("{:?} {}", scenes, app.framerate().unwrap());

    Ok(scenes)
}

pub fn split_video(term: &mut Terminal<impl Backend>, app: &mut App, scenes: Vec<RangeInclusive<usize>>) -> io::Result<()> {
    // make sure output dir exists
    // split scene
    // progress split on app
    // info about split
    // draw app
    
    Ok(())
}

pub fn cleanup(term: &mut Terminal<impl Backend>) -> io::Result<()> {
    std::thread::sleep(std::time::Duration::from_secs(22));

    term.clear()?;
    term.show_cursor()?;

    Ok(())
}
