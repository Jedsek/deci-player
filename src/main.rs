#![allow(clippy::single_match)]

mod audio;
mod config;
mod handle;
mod view;

use std::time::Duration;

use config::Config;
use handle::handle_key;
use iced::advanced::graphics::core::SmolStr;
use iced::keyboard::{Key, Modifiers};
use iced::{executor, keyboard, window, Application, Command, Size, Subscription};
use iced::{Element, Font, Settings, Theme};
use rodio::{Sink, Source};

fn main() -> iced::Result {
    App::run(Settings {
        fonts: vec![include_bytes!("../MapleMono-NF-CN-Regular.ttf").into()],
        default_font: Font::with_name("Maple Mono NF CN"),
        // fonts: vec![include_bytes!("../SarasaMonoTC-Regular.ttf").into()],
        // default_font: Font::with_name("Sarasa Mono TC"),
        antialiasing: true,
        window: window::Settings {
            size: Size::new(1280.0, 720.0),
            ..Default::default()
        },
        ..Default::default()
    })
}

struct App {
    is_prev_playing: bool,
    mode: ViewMode,
    config: Vec<Config>,
    sink: Sink,
    current_pos: usize,
    current_source: audio::TheSource,
    slider_value: f32,
    time: Duration,
    lang: Lang,
    tick_secs: f32,
    speed: f32,
}

impl Default for App {
    fn default() -> Self {
        let sink = audio::new_sink();
        let config = Config::new("./config.toml");
        let current_pos = 0;
        let mode = ViewMode::Play;
        let lang = Lang::All;

        let source_path = &config[current_pos].source_path;
        sink.append(audio::new_source(source_path));
        let current_source = audio::new_source(source_path);

        let time = Duration::ZERO;
        let slider_value = 0.0;
        let tick_secs = 0.1;
        let speed = 1.0;

        // audio::sample(current_source.clone());

        for (idx, i) in config.iter().enumerate() {
            if idx != current_pos {
                let source = audio::new_source(&i.source_path);
                sink.append(source);
            }
        }

        // audio::sample(audio::new_source(source_path));

        Self {
            is_prev_playing: true,
            mode,
            config,
            sink,
            current_pos,
            current_source,
            time,
            lang,
            slider_value,
            tick_secs,
            speed,
        }
    }
}

impl App {
    fn switch_view(&mut self, mode: ViewMode) {
        self.mode = mode;
    }

    fn update_time(&mut self) {
        if !self.sink.is_paused() {
            self.time += Duration::from_secs_f32(self.tick_secs * self.speed);
            self.slider_value += self.tick_secs;
        }

        if !self.sink.is_paused() && self.time >= self.current_source.total_duration().unwrap() {
            self.next_song();
        }
    }

    fn skip_song(&self) {
        let sink = &self.sink;
        let source_path = &self.config[self.current_pos].source_path;

        sink.append(audio::new_source(source_path));
    }

    fn next_pos(&self) -> usize {
        let max_pos = self.config.len() - 1;
        if self.current_pos >= max_pos {
            0
        } else {
            self.current_pos + 1
        }
    }

    fn next_song(&mut self) {
        self.skip_song();
        self.sink.play();
        self.current_pos = self.next_pos();
        self.time = Duration::ZERO;
        self.slider_value = 0.0;

        let source_path = &self.config[self.current_pos].source_path;
        self.current_source = audio::new_source(source_path);
    }

    // fn prev_pos(&self) -> usize {
    //     let max_pos = self.config.len() - 1;
    //     if self.current_pos == 0 {
    //         max_pos
    //     } else {
    //         self.current_pos - 1
    //     }
    // }

    // fn prev_song(&mut self) {
    //     self.skip_song();
    //     self.current_pos = self.prev_pos();
    //     self.time = Duration::ZERO;
    //     self.slider_value = 0.0;

    //     let source_path = &self.config[self.current_pos].source_path;
    //     self.current_source = audio::new_source(source_path);
    // }

    fn toggle_play(&mut self) {
        let sink = &self.sink;
        if sink.is_paused() {
            sink.play();
        } else {
            sink.pause()
        }
        self.is_prev_playing = !sink.is_paused();
    }

    fn set_volume(&self, relative_factor: i8) {
        let sink = &self.sink;
        let volume = ((sink.volume() * 100.0) as i8 + relative_factor).clamp(0, 100);
        sink.set_volume(volume as f32 / 100.0);
    }

    fn seek_audio(&mut self) {
        self.time = Duration::from_secs_f32(self.slider_value);
        self.sink.try_seek(self.time).unwrap();
    }

    fn toggle_lang(&mut self) {
        self.lang = match self.lang {
            Lang::Chinese => Lang::Japanese,
            Lang::Japanese => Lang::All,
            Lang::All => Lang::Chinese,
            _ => unimplemented!(),
        }
    }

    fn toggle_speed(&mut self) {
        self.speed = match self.speed {
            0.5 => 1.0,
            1.0 => 1.5,
            1.5 => 2.0,
            2.0 => 0.5,
            _ => unreachable!(),
        };
        self.sink.set_speed(self.speed);
    }
}

impl Application for App {
    type Executor = executor::Default;
    type Flags = ();
    type Message = Message;
    type Theme = Theme;

    fn new(_flags: ()) -> (Self, Command<Self::Message>) {
        let app = Self::default();
        (app, Command::none())
    }

    fn title(&self) -> String {
        "Music player - Iced".into()
    }

    fn theme(&self) -> Theme {
        Theme::Nord
    }

    fn subscription(&self) -> Subscription<Self::Message> {
        let key = keyboard::on_key_press(|key, modifiers| {
            let msg = Message::KeyInput { key, modifiers };
            Some(msg)
        });

        let time =
            iced::time::every(Duration::from_secs_f32(self.tick_secs)).map(|_| Message::UpdateTime);

        Subscription::batch([key, time])
    }

    fn update(&mut self, msg: Message) -> Command<Self::Message> {
        match msg {
            Message::TogglePlay => self.toggle_play(),
            Message::ToggleLang => self.toggle_lang(),
            Message::SetVolume(factor) => self.set_volume(factor),
            Message::NextSong => {
                self.next_song();
                self.sink.skip_one()
            }
            // Message::PrevSong => self.prev_song(),
            Message::SwitchView(mode) => {
                if mode == ViewMode::Play && self.is_prev_playing {
                    self.sink.play()
                } else {
                    self.sink.pause()
                }
                self.switch_view(mode)
            }
            Message::Quit => {
                std::process::exit(0);
            }
            Message::KeyInput { key, modifiers } => {
                let msg = handle_key(self.mode, key, modifiers);
                return self.update(msg);
            }
            Message::UpdateTime => self.update_time(),
            Message::SeekAudio => self.seek_audio(),
            Message::UpdateSlider(val) => self.slider_value = val,
            Message::ToggleSpeed => self.toggle_speed(),
            _ => (),
        };

        Command::none()
    }

    fn view(&self) -> Element<Message> {
        match self.mode {
            ViewMode::Play => view::play(self),
            ViewMode::Help => view::help(self),
            ViewMode::ConfirmQuit => view::confirm_quit(self),
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum ViewMode {
    Play,
    Help,
    ConfirmQuit,
}

#[derive(Debug, Clone)]
enum Message {
    Nothing,
    TogglePlay,
    ToggleLang,
    SetVolume(i8),
    NextSong,
    PrevSong,
    SwitchView(ViewMode),
    KeyInput {
        key: Key<SmolStr>,
        modifiers: Modifiers,
    },
    Quit,
    UpdateSlider(f32),
    ToggleSpeed,
    UpdateTime,
    SeekAudio,
}

#[derive(Debug, Clone, Copy)]
enum Lang {
    All,
    Chinese,
    Japanese,
    #[allow(dead_code)]
    English,
}
