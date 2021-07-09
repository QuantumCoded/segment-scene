mod app;
mod util;

#[macro_use]
extern crate clap;

use app::App;
use clap::Arg;
use std::io::{self, stdout};
use tui::{backend::CrosstermBackend, terminal::Terminal};

fn main() -> io::Result<()> {
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

    let input = matches.value_of("INPUT").unwrap();
    let scale: f64 = matches
        .value_of("scale")
        .unwrap()
        .parse()
        .expect("could not parse scale value");
    let threshold: f64 = matches
        .value_of("threshold")
        .unwrap()
        .parse()
        .expect("could not parse threshold value");
    let lookahead: u8 = matches
        .value_of("lookahead")
        .unwrap()
        .parse()
        .expect("could not parse lookahead value");
    let _format = matches.value_of("format").unwrap();
    let _slideshow = matches.is_present("slideshow");
    let _keep_cache = matches.is_present("keepcache");

    assert!(lookahead >= 1);

    let stdout = stdout();
    let backend = CrosstermBackend::new(stdout);
    let mut term = Terminal::new(backend)?;

    term.clear()?;

    let mut app = App::new(input);

    util::get_framerate(&mut term, &mut app)?;
    util::get_frame_count(&mut term, &mut app)?;
    std::thread::sleep(std::time::Duration::from_secs(2));
    util::create_image_sequence(&mut term, &mut app, scale)?;
    let scenes = util::compare_frames(&mut term, &mut app, threshold, lookahead)?;
    util::split_video(&mut term, &mut app, scenes)?;
    util::cleanup(&mut term)?;

    Ok(())
}
