use crate::apps::app::{ClickableArea};
use crate::drivers::ili9341::{HEIGHT, WIDTH};
use crate::phone::PhoneData;
use crate::ui::widgets::keyboard::KeyboardEvent::*;
use mousefood::prelude::{Color, Frame, Rect, Stylize};
use mousefood::ratatui::widgets::{Block, Paragraph};
use once_cell::sync::Lazy;

pub const KEYBOARD_HEIGHT: u16 = 12;

static AZERTY: Lazy<[Vec<(&'static str, KeyboardEvent, u16)>;4]> = Lazy::new(|| [
    vec![("a", Letter('a'), 0), ("z", Letter('z'), 3), ("e", Letter('e'), 6), ("r", Letter('r'), 9), ("t", Letter('t'), 12), ("y", Letter('y'), 15), ("u", Letter('u'), 18), ("i", Letter('i'), 21), ("o", Letter('o'), 24), ("p", Letter('p'), 27)],
    vec![("q", Letter('q'), 0), ("s", Letter('s'), 3), ("d", Letter('d'), 6), ("f", Letter('f'), 9), ("g", Letter('g'), 12), ("h", Letter('h'), 15), ("j", Letter('j'), 18), ("k", Letter('k'), 21), ("l", Letter('l'), 24), ("m", Letter('m'), 27)],
    vec![("Maj", Maj, 0), ("w", Letter('w'), 5), ("x", Letter('x'), 8), ("c", Letter('c'), 11), ("v", Letter('v'), 14), ("b", Letter('b'), 17), ("n", Letter('n'), 20), ("'", Letter('\''), 23), ("DEL", Delete, 26)],
    vec![("?123", Symbols(SymbolLevel::First), 0), (",", Letter(','), 6), ("          ", Letter('x'), 9), (".", Letter('.'), 21), ("ENTER", Enter, 24)],
]);

static SYMBOLS_1: Lazy<[Vec<(&'static str, KeyboardEvent, u16)>;4]> = Lazy::new(|| [
    vec![("1", Letter('1'), 0), ("2", Letter('2'), 3), ("3", Letter('3'), 6), ("4", Letter('4'), 9), ("5", Letter('5'), 12), ("6", Letter('6'), 15), ("7", Letter('7'), 18), ("8", Letter('8'), 21), ("9", Letter('9'), 24), ("0", Letter('0'), 27)],
    vec![("@", Letter('@'), 0), ("#", Letter('#'), 3), ("€", Letter('€'), 6), ("_", Letter('_'), 9), ("&", Letter('&'), 12), ("-", Letter('-'), 15), ("+", Letter('+'), 18), ("(", Letter('('), 21), (")", Letter(')'), 24), ("/", Letter('/'), 27)],
    vec![("=\\<", Symbols(SymbolLevel::Second), 0), ("*", Letter('*'), 5), ("\"", Letter('"'), 8), ("'", Letter('\''), 11), (":", Letter(':'), 14), (";", Letter(';'), 17), ("!", Letter('!'), 20), ("?", Letter('?'), 23), ("DEL", Delete, 26)],
    vec![("?123", Symbols(SymbolLevel::None), 0), (",", Letter(','), 6), ("          ", Letter('x'), 9), (".", Letter('.'), 21), ("ENTER", Enter, 24)],
]);

static SYMBOLS_2: Lazy<[Vec<(&'static str, KeyboardEvent, u16)>;4]> = Lazy::new(|| [
    vec![("~", Letter('~'), 0), ("`", Letter('`'), 3), ("|", Letter('|'), 6), ("•", Letter('•'), 9), ("√", Letter('√'), 12), ("π", Letter('π'), 15), ("÷", Letter('÷'), 18), ("×", Letter('×'), 21), ("§", Letter('§'), 24), ("∆", Letter('∆'), 27)],
    vec![("£", Letter('£'), 0), ("¥", Letter('¥'), 3), ("$", Letter('$'), 6), ("¢", Letter('¢'), 9), ("^", Letter('^'), 12), ("°", Letter('°'), 15), ("=", Letter('='), 18), ("{", Letter('{'), 21), ("}", Letter('}'), 24), ("\\", Letter('\\'), 27)],
    vec![("=\\<", Symbols(SymbolLevel::First), 0), ("%", Letter('%'), 5), ("©", Letter('©'), 8), ("®", Letter('®'), 11), ("™", Letter('™'), 14), ("✓", Letter('✓'), 17), ("[", Letter('['), 20), ("]", Letter(']'), 23), ("DEL", Delete, 26)],
    vec![("?123", Symbols(SymbolLevel::None), 0), ("<", Letter('<'), 6), ("          ", Letter('x'), 9), (">", Letter('>'), 21), ("ENTER", Enter, 24)],
]);

static AZERTY_EVENTS: Lazy<Vec<PreRenderedParagraph>> = Lazy::new(|| prerender_layout(&*AZERTY, false));
static AZERTY_MAJ_EVENTS: Lazy<Vec<PreRenderedParagraph>> = Lazy::new(|| prerender_layout(&*AZERTY, true));
static SYMBOLS_1_EVENTS: Lazy<Vec<PreRenderedParagraph>> = Lazy::new(|| prerender_layout(&*SYMBOLS_1, false));
static SYMBOLS_2_EVENTS: Lazy<Vec<PreRenderedParagraph>> = Lazy::new(|| prerender_layout(&*SYMBOLS_2, false));

struct PreRenderedParagraph<'a> {
    pub paragraph: Paragraph<'a>,
    pub area: Rect,
    pub event: Box<KeyboardEvent>
}

#[derive(Debug, Clone)]
pub struct Keyboard {
    pub text: String,
    layout: KeyboardLayout,
    maj: bool,
    symbols: SymbolLevel,
    hide_enter: bool,
}

#[derive(Debug, Clone)]
pub enum KeyboardLayout {
    Azerty
}

#[derive(Debug, Copy, Clone, PartialEq)]
pub enum KeyboardEvent {
    Letter(char),
    Maj,
    Enter,
    Delete,
    Symbols(SymbolLevel),
    None
}

#[derive(Debug, Copy, Clone, PartialEq)]
pub enum SymbolLevel {
    None,
    First,
    Second,
}

impl KeyboardEvent {
    fn maj_letter(&self) -> Self {
        match self {
            Letter(letter) => Letter(letter.to_ascii_uppercase()),
            other => *other
        }
    }
}

impl Keyboard {
    pub fn render(&self, frame: &mut Frame) -> Vec<ClickableArea> {
        let mut events = Vec::new();

        let pre_rendered_keyboard = match self.symbols {
            SymbolLevel::None => match self.layout {
                KeyboardLayout::Azerty => match self.maj {
                    false => &*AZERTY_EVENTS,
                    true => &*AZERTY_MAJ_EVENTS,
                }
            },
            SymbolLevel::First => &*SYMBOLS_1_EVENTS,
            SymbolLevel::Second => &*SYMBOLS_2_EVENTS
        };

        for pre_rendered_paragraph in pre_rendered_keyboard {
            let (paragraph, event) = match self.hide_enter && *pre_rendered_paragraph.event == Enter {
                true => (pre_rendered_paragraph.paragraph.clone().fg(Color::DarkGray), Box::new(None)),
                false => (pre_rendered_paragraph.paragraph.clone(), pre_rendered_paragraph.event.clone())
            };

            frame.render_widget(paragraph, pre_rendered_paragraph.area);

            events.push(ClickableArea(pre_rendered_paragraph.area, event));
        }

        events
    }

    pub fn handle_event(&mut self, event: &KeyboardEvent) {
        match event {
            Letter(letter) => self.text.push(*letter),
            Maj => {
                self.maj = !self.maj;
                self.symbols = SymbolLevel::None;
            },
            Delete => {
                self.text.pop();
            },
            Enter => self.text.push('\n'),
            Symbols(level) => self.symbols = *level,
            None => {}
        }
    }
}

impl PhoneData {
    pub fn display_keyboard(&mut self, layout: KeyboardLayout, hide_enter: bool) {
        self.keyboard = Some(Keyboard {
            text: String::new(),
            layout,
            maj: false,
            symbols: SymbolLevel::None,
            hide_enter,
        })
    }

    pub fn hide_keyboard(&mut self) {
        self.keyboard = Option::None;
    }
}

fn prerender_layout(keyboard_layout: &[Vec<(&'static str, KeyboardEvent, u16)>; 4], uppercase: bool) -> Vec<PreRenderedParagraph<'static>> {
    let area = Rect {
        x: 0,
        y: HEIGHT - KEYBOARD_HEIGHT,
        width: WIDTH,
        height: KEYBOARD_HEIGHT,
    };

    let mut pre_rendered = Vec::new();

    for (row_index, row) in keyboard_layout.iter().enumerate() {
        for (text, event, x) in row {
            let (text, event) = match uppercase {
                true => (text.to_ascii_uppercase(), event.maj_letter()),
                false => (text.to_string(), event.clone())
            };

            let rect = Rect {
                x: area.x + 4 + x,
                y: area.y + (row_index as u16 * 3),
                width: 2 + text.chars().count() as u16,
                height: 3,
            };

            let paragraph = Paragraph::new(text)
                .fg(Color::White)
                .block(Block::bordered().fg(Color::DarkGray));

            pre_rendered.push(PreRenderedParagraph {
                paragraph,
                area: rect,
                event: Box::new(event)
            });
        }
    }

    pre_rendered
}