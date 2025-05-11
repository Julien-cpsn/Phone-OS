use mousefood::prelude::{Buffer, Line, Rect, Stylize, Widget};
use mousefood::ratatui::widgets::{Block, Borders};

pub struct BorderedButton<'a>(pub &'a str);

impl<'a> Widget for BorderedButton<'a> {
    fn render(self, area: Rect, buf: &mut Buffer) where Self: Sized {
        let block = Block::default()
            .borders(Borders::ALL);
        
        let inner = block.inner(area);
        
        let span = Line::raw(self.0).centered().white();
        
        block.render(area, buf);
        span.render(inner, buf);
    }
}