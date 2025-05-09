use mousefood::prelude::{Frame, Rect, Style, Stylize};
use mousefood::ratatui::widgets::{Block, List, Paragraph};
use crate::phone::Phone;
use crate::drivers::ft6206::TouchPoint;

const WIDTH: u16 = 24;
const HEIGHT: u16 = 32;

impl<'a> Phone<'a> {
    pub fn draw(&mut self, frame: &mut Frame, events: Vec<TouchPoint>) {
        let bordered_block = Block::bordered()
            .border_style(Style::new().yellow())
            .title("Phone OS");
        let apps = List::new(["Mon app", "Mon autre app"])
            .block(bordered_block);
        frame.render_widget(apps, frame.area());

        for event in events {
            let marker = Paragraph::new("X").yellow();
            let mut x = WIDTH - (event.x * WIDTH / 240);
            let mut y = HEIGHT - 1 - (event.y * HEIGHT / 320);

            if x >= WIDTH {
                x = WIDTH - 1;
            }
            if y >= HEIGHT {
                y = HEIGHT - 1;
            }
            let touch_area = Rect::new(x, y, 1, 1);

            frame.render_widget(marker, touch_area);
        }
    }

    pub fn draw_homepage(&mut self, frame: &mut Frame) {
        let logo = Paragraph::new(vec![
            "_____  _".into(),
            "|  __ \\| |".into(),
            "| |__) | |__   ___  _ __   ___".into(),
            "|  ___/| '_ \\ / _ \\| '_ \\ / _ \\".into(),
            "| |    | | | | (_) | | | |  __/".into(),
            "|_|___ |_|_|_|\\___/|_| |_|\\___|".into(),
            "/ __ \\ / ____|".into(),
            "| |  | | (___".into(),
            "| |  | |\\__ \\".into(),
            "| |__| |___) |".into(),
            "\\____/|_____/".into(),
            "Loading...".into(),
        ]);
        frame.render_widget(logo, frame.area());
    }
}

/*
_____  _
|  __ \| |
| |__) | |__   ___  _ __   ___
|  ___/| '_ \ / _ \| '_ \ / _ \
| |    | | | | (_) | | | |  __/
|_|___ |_|_|_|\___/|_| |_|\___|
/ __ \ / ____|
| |  | | (___
| |  | |\___ \
| |__| |____) |
\____/|_____/
*/