use std::path::{Path, PathBuf};
use tui::layout::{Constraint, Direction, Layout};
use tui::style::{Color, Style};
use tui::{backend::Backend, symbols::Marker, widgets::*, Frame};

pub enum ProgressMode {
    Comparing,
    Splitting,
}

pub struct App {
    input: PathBuf,
    cache: Option<PathBuf>,
    dssim_data: Vec<(f64, f64)>,
    median_data: Vec<(f64, f64)>,
    threshold_data: Vec<(f64, f64)>,
    info: Vec<String>,
    progress_mode: ProgressMode,
    progress: usize,
    compares: Option<usize>,
    scenes: Option<usize>,
    framerate: Option<f64>,
}

impl App {
    pub fn new(path: impl AsRef<Path>) -> App {
        App {
            input: path.as_ref().to_owned(),
            cache: None,
            dssim_data: Vec::new(),
            median_data: Vec::new(),
            threshold_data: Vec::new(),
            info: Vec::new(),
            progress_mode: ProgressMode::Comparing,
            progress: 0,
            compares: None,
            scenes: None,
            framerate: None,
        }
    }

    pub fn input(&self) -> &Path {
        self.input.as_ref()
    }

    pub fn cache(&self) -> Option<&Path> {
        match &self.cache {
            Some(cache) => Some(cache.as_ref()),
            None => None,
        }
    }

    pub fn set_cache(&mut self, path: impl AsRef<Path>) {
        if self.cache.is_none() {
            self.cache = Some(path.as_ref().to_owned());
        }
    }

    pub fn info(&mut self, info: impl AsRef<str>) {
        self.info.push(info.as_ref().to_owned());
    }

    pub fn progress_compare(&mut self, threshold: f64, diff: f64) {
        if let ProgressMode::Comparing = self.progress_mode {
            self.progress += 1;

            let progress = self.progress as f64;

            self.dssim_data.push((progress, diff));
            self.threshold_data.push((progress, threshold));
        }
    }

    pub fn progress_split(&mut self) {
        match self.progress_mode {
            ProgressMode::Comparing => {
                self.progress_mode = ProgressMode::Splitting;
                self.progress = 1;
            }

            ProgressMode::Splitting => {
                self.progress += 1;
            }
        }
    }

    pub fn get_progress(&self) -> usize {
        self.progress
    }

    pub fn framerate(&self) -> Option<f64> {
        self.framerate
    }

    pub fn set_framerate(&mut self, fr: f64) {
        if self.framerate.is_none() {
            self.framerate = Some(fr);
        }
    }

    pub fn set_frame_count(&mut self, fc: usize) {
        if self.compares.is_none() {
            self.compares = Some(fc - 1);
        }
    }

    pub fn set_scene_count(&mut self, sc: usize) {
        if self.scenes.is_none() {
            self.scenes = Some(sc);
        }
    }

    pub fn draw(&self, f: &mut Frame<impl Backend>) {
        let chunks = Layout::default()
            .direction(Direction::Vertical)
            .constraints([
                Constraint::Min(0),
                Constraint::Length(9),
                Constraint::Length(3),
            ])
            .split(f.size());

        let data_block = Block::default().title("data").borders(Borders::ALL);
        let info_block = Block::default().title("info").borders(Borders::ALL);
        let prog_block = Block::default()
            .title(match self.progress_mode {
                ProgressMode::Comparing => "comparing",
                ProgressMode::Splitting => "splitting",
            })
            .borders(Borders::ALL);

        let width = data_block.inner(chunks[0]).width;

        let data_chart = Chart::new(vec![
            Dataset::default()
                .marker(Marker::Dot)
                .style(Style::default().fg(Color::Rgb(255, 0, 0)).bg(Color::Reset))
                .data(&self.dssim_data),
            Dataset::default()
                .marker(Marker::Braille)
                .style(Style::default().fg(Color::Cyan))
                .data(&self.median_data),
            Dataset::default()
                .marker(Marker::Dot)
                .style(Style::default().fg(Color::Rgb(0, 0, 255)))
                .data(&self.threshold_data),
        ])
        .block(data_block)
        .x_axis(Axis::default().bounds([
            0.,
            self.progress as f64,
        ]))
        .y_axis(Axis::default().bounds([0., 1.5]));

        let info: Vec<ListItem> = self
            .info
            .iter()
            .map(|item| ListItem::new(item.as_ref()))
            .collect();
        let info_list = List::new(info).block(info_block);
        let mut state = ListState::default();
        state.select(Some(self.info.len() - 1));

        let ratio = match self.compares {
            None => 0.,
            Some(frames) => self.progress as f64 / frames as f64,
        };

        let label = format!("{}/{}", self.progress, match self.progress_mode {
            ProgressMode::Comparing => self.compares.unwrap_or_default(),
            ProgressMode::Splitting => self.scenes.unwrap_or_default(),
        });
        let prog_gauge = Gauge::default()
            .block(prog_block)
            .gauge_style(Style::default().fg(Color::Indexed(4)))
            .ratio(ratio)
            .label(label);

        f.render_widget(data_chart, chunks[0]);
        f.render_stateful_widget(info_list, chunks[1], &mut state);
        f.render_widget(prog_gauge, chunks[2]);
    }
}
