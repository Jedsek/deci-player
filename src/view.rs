mod utils;

use std::{fs, sync::OnceLock};

use crate::{audio, config::Config, App, Lang, Message};
use iced::{
    alignment::Horizontal,
    theme::{self, TextEditor},
    widget::{column, image, radio::StyleSheet, row, text, Column, Container, ProgressBar, Slider},
    Alignment, Color, Element, Length,
};
use iced_aw::floating_element;
use iced_aw::floating_element::Anchor;
use itertools::Itertools;
use rodio::Source;

const KEY_DESCRIPTION: &[(&str, &str)] = &[
    ("\n● 模式/播放", "\n"),
    ("h", "进入帮助页面"),
    ("[p, space]", "播放/暂停"),
    ("t", "切换语言(默认双语字幕, 每次切换至中文/日语/双语)"),
    ("q", "关闭应用"),
    ("\n\n● 模式/帮助", "\n"),
    ("h", "退出帮助页面"),
    ("\n\n● 模式/退出", "\n"),
    ("y", "确认"),
    ("n", "取消"),
];

const TITLE_SIZE: u16 = 36;
const TITLE_PADDING: u16 = 20;
const CONTENT_SIZE: u16 = 20;

pub fn play(app: &App) -> Element<Message> {
    let Config {
        name,
        avatar,
        background,
        ..
    } = &app.config[app.current_pos];

    let background = utils::background_image(background);

    let avatar = image(avatar).width(400).height(400);
    let name = text(name)
        .size(40)
        // .width(Length::Fill)
        // .horizontal_alignment(Horizontal::Center)
        .style(utils::text(utils::black()));
    let status_line = status_line(app);

    let lyric = match app.lang {
        Lang::All => {
            let lyric_1 = get_lyrics(app, Lang::Chinese).unwrap_or_default();
            let lyric_1 = text(lyric_1).size(30).style(utils::text(utils::black()));

            let lyric_2 = get_lyrics(app, Lang::Japanese).unwrap_or_default();
            let lyric_2 = text(lyric_2).size(30).style(utils::text(utils::black()));
            column!(lyric_1, lyric_2)
        }
        Lang::Chinese => {
            let lyric = get_lyrics(app, Lang::Chinese).unwrap_or_default();
            let lyric = text(lyric).size(30).style(utils::text(utils::black()));
            column!(lyric)
        }
        Lang::Japanese => {
            let lyric = get_lyrics(app, Lang::Japanese).unwrap_or_default();
            let lyric = text(lyric).size(30).style(utils::text(utils::black()));
            column!(lyric)
        }
        _ => unimplemented!(),
    };

    let lyric = lyric.padding(40).align_items(Alignment::Center);

    let total_duration = get_total_duration(app);
    let slider = Slider::new(
        0.0..=total_duration,
        app.slider_value,
        Message::UpdateSlider,
    )
    .on_release(Message::SeekAudio)
    .height(15)
    .width(600)
    .style(theme::Slider::Custom(Box::new(utils::StyledSlider)));

    let right = column!(name, status_line, slider, lyric)
        .spacing(5)
        .width(Length::Fill)
        .align_items(Alignment::Center);

    // let right = Container::new(right).center_x();

    let container = row!(avatar, right);
    let container = floating_element(background, container)
        .anchor(Anchor::NorthWest)
        .offset([90.0, 150.0]);

    container.into()
}

pub fn help(_app: &App) -> Element<Message> {
    let title = text("Help")
        .width(Length::Shrink)
        .size(TITLE_SIZE)
        .horizontal_alignment(Horizontal::Center)
        .style(utils::text(utils::cyan()));
    let title = row!(title).padding(TITLE_PADDING);

    let mut content = Column::new().padding(5);
    for (key, desc) in get_help_text() {
        let to_text = |s| {
            text(s)
                .width(Length::Shrink)
                .size(CONTENT_SIZE)
                .style(utils::text(utils::cyan()))
        };
        let (key, desc) = (to_text(key), to_text(desc));
        let row = row!(key, desc).spacing(50).padding(2);
        content = content.push(row);
    }
    let content = Container::new(content).width(Length::Fill).center_x();

    let container = column!(title, content).spacing(20);
    let container = Container::new(container).width(Length::Shrink).center_x();

    container.into()
}

pub fn confirm_quit(_app: &App) -> Element<Message> {
    let title = text("Quit?")
        .width(Length::Shrink)
        .size(TITLE_SIZE)
        .horizontal_alignment(Horizontal::Center)
        .style(utils::text(utils::cyan()));
    let title = row!(title).padding(TITLE_PADDING);

    let tips = text("Y / N")
        .width(Length::Shrink)
        .size(CONTENT_SIZE * 2)
        .style(utils::text(utils::cyan()));
    let tips = Container::new(tips).width(Length::Fill).center_x();

    let container = column!(title, tips).spacing(20);
    let container = Container::new(container).width(Length::Shrink).center_x();

    container.into()
}

pub fn status_line(app: &App) -> Element<Message> {
    let volume = format!("音量: {}%", (app.sink.volume() * 100.0) as i8);
    let volume = text(volume).size(20).style(utils::text(utils::black()));

    let (minute, second) = {
        let time = app.time.as_secs();
        (time / 60, time % 60)
    };
    let (total_minute, total_second) = {
        let total_time = app.current_source.total_duration().unwrap().as_secs();
        (total_time / 60, total_time % 60)
    };

    let time = if minute == 0 {
        format!("已播放: {}s/{}m{}s", second, total_minute, total_second,)
    } else {
        format!(
            "已播放: {}m{}s/{}m{}s",
            minute, second, total_minute, total_second,
        )
    };

    let time = text(time).size(20).style(utils::text(utils::black()));
    let is_paused = text(if app.sink.is_paused() {
        "暂停中"
    } else {
        "播放中"
    })
    .size(20)
    .style(utils::text(utils::black()));

    row!(time, volume, is_paused).spacing(30).into()
}

fn get_help_text() -> &'static Vec<(String, String)> {
    static KEY_DESCRIPTION_CACHE: OnceLock<Vec<(String, String)>> = OnceLock::new();

    KEY_DESCRIPTION_CACHE.get_or_init(|| {
        let get_len = |s: &str| {
            s.chars()
                .fold(0, |acc, ch| acc + if ch.is_ascii() { 1 } else { 2 })
        };

        let get_format = |s: &str, max_len: usize| {
            let count = max_len - get_len(s);
            String::from(s) + " ".repeat(count).as_str()
        };

        let (mut key_max_len, mut desc_max_len) = (0, 0);
        for (key, desc) in KEY_DESCRIPTION {
            key_max_len = get_len(key).max(key_max_len);
            desc_max_len = get_len(desc).max(desc_max_len);
        }

        KEY_DESCRIPTION
            .iter()
            .map(|(key, desc)| {
                let key = get_format(key, key_max_len);
                let desc = get_format(desc, desc_max_len);
                (key, desc)
            })
            .collect()
    })
}

fn get_total_duration(app: &App) -> f32 {
    app.current_source.total_duration().unwrap().as_secs_f32()
}

fn get_lyrics(app: &App, lang: Lang) -> Option<String> {
    let config = &app.config;
    let pos = app.current_pos;

    let path = match lang {
        Lang::Chinese => config[pos].lyrics_first.as_ref(),
        Lang::Japanese => config[pos].lyrics_second.as_ref(),
        _ => unimplemented!(),
    };

    let path = path.unwrap();
    let lyric = fs::read_to_string(path).unwrap();
    let duration = app.time.as_secs_f32();

    let mut s = "";
    let result = lyric.lines().rev().find(|line| {
        let pos = line.chars().positions(|v| "[:]".contains(v)).collect_vec();

        if pos.len() < 3 || !(line[1..=1].chars().collect_vec()[0]).is_ascii_digit() {
            return false;
        }

        let minute = &line[(pos[0] + 1)..pos[1]];
        let second = &line[(pos[1] + 1)..pos[2]];
        s = &line[(pos[2] + 1)..];

        let time = minute
            .parse::<f32>()
            .and_then(|m| second.parse::<f32>().map(|s| m * 60.0 + s))
            .ok();

        time.is_some() && !s.is_empty() && duration >= time.unwrap()
    });
    result.map(|_| s.to_string().replace(['，', '。'], " "))
}
