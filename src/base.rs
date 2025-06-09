use iter_identify_first_last::IteratorIdentifyFirstLastExt;
use serde::{Serialize, Deserialize};

pub use int_vec_2d::*;
pub use tvxaml_screen_base::*;

pub fn label_width(text: &str) -> i16 {
    let mut width = 0i16;
    let mut hotkey = false;
    for (first, last, text) in text.split('~').identify_first_last() {
        if !first && !text.is_empty() {
            hotkey = !hotkey;
        }
        let actual_text = if !first && !last && text.is_empty() { "~" } else { text };
        width = width.wrapping_add(text_width(actual_text));
        if !first && text.is_empty() {
            hotkey = !hotkey;
        }
    }
    width
}

pub fn label(text: &str) -> Option<char> {
    let mut hotkey = false;
    for (first, last, text) in text.split('~').identify_first_last() {
        if !first && !text.is_empty() {
            hotkey = !hotkey;
        }
        let actual_text = if !first && !last && text.is_empty() { "~" } else { text };
        if hotkey && !actual_text.is_empty() {
            return Some(actual_text.chars().next().unwrap().to_lowercase().next().unwrap());
        }
        if !first && text.is_empty() {
            hotkey = !hotkey;
        }
    }
    None
}

#[derive(Clone, Copy, Eq, PartialEq, Ord, PartialOrd, Hash)]
#[derive(Serialize, Deserialize)]
pub enum TextWrapping {
    NoWrap,
    Wrap,
    WrapWithOverflow,
}

#[derive(Debug, Copy, Clone, Eq, PartialEq, Ord, PartialOrd, Hash)]
#[derive(Serialize, Deserialize)]
pub enum TextAlign { Left, Center, Right, Justify }

impl From<Option<HAlign>> for TextAlign {
    fn from(value: Option<HAlign>) -> Self {
        match value {
            Some(HAlign::Left) => TextAlign::Left,
            Some(HAlign::Center) => TextAlign::Center,
            Some(HAlign::Right) => TextAlign::Right,
            None => TextAlign::Justify,
        }
    }
}

impl From<TextAlign> for Option<HAlign> {
    fn from(value: TextAlign) -> Self {
        match value {
            TextAlign::Left => Some(HAlign::Left),
            TextAlign::Center => Some(HAlign::Center),
            TextAlign::Right => Some(HAlign::Right),
            TextAlign::Justify => None,
        }
    }
}
