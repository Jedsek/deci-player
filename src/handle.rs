use iced::{
    advanced::graphics::core::SmolStr,
    keyboard::{key::Named, Key, Modifiers},
};

use crate::{Message, ViewMode};

pub fn handle_key(mode: ViewMode, key: Key<SmolStr>, modifiers: Modifiers) -> Message {
    let key = key.as_ref();

    match mode {
        ViewMode::Play => handle_in_normal(key, modifiers),
        ViewMode::Help => handle_in_help(key, modifiers),
        ViewMode::ConfirmQuit => handle_in_confirm_quit(key, modifiers),
    }
}

fn handle_in_confirm_quit(key: Key<&str>, _modifiers: Modifiers) -> Message {
    if let Key::Character(c) = key {
        match c {
            "y" => Message::Quit,
            "n" => Message::SwitchView(ViewMode::Play),
            _ => Message::Nothing,
        }
    } else {
        Message::Nothing
    }
}

fn handle_in_normal(key: Key<&str>, _modifiers: Modifiers) -> Message {
    if let Key::Character(c) = key {
        match c {
            "h" => Message::SwitchView(ViewMode::Help),
            "q" => Message::SwitchView(ViewMode::ConfirmQuit),
            "p" => Message::TogglePlay,
            "t" => Message::ToggleLang,
            _ => Message::Nothing,
        }
    } else if let Key::Named(n) = key {
        match n {
            Named::ArrowUp => Message::SetVolume(10),
            Named::ArrowDown => Message::SetVolume(-10),
            Named::ArrowRight => Message::NextSong,
            Named::ArrowLeft => Message::PrevSong,
            Named::Space => Message::TogglePlay,
            _ => Message::Nothing,
        }
    } else {
        Message::Nothing
    }
}

fn handle_in_help(key: Key<&str>, _modifiers: Modifiers) -> Message {
    if let Key::Character(c) = key {
        match c {
            "h" => Message::SwitchView(ViewMode::Play),
            _ => Message::Nothing,
        }
    } else {
        Message::Nothing
    }
}
