use crate::client::redux::State;
use ratatui::backend::Backend;
use ratatui::layout::{Alignment, Constraint, Direction, Layout};
use ratatui::style::{Color, Style};
use ratatui::text::{Span, Spans};
use ratatui::widgets::{Block, BorderType, Borders, Wrap};
use ratatui::Frame;

#[derive(Default)]
pub struct AppComponent {}

impl AppComponent {
    pub fn draw<B: Backend>(&self, f: &mut Frame<B>, state: State) {
        let fsize = f.size();

        let chunks_main = Layout::default()
            .direction(Direction::Vertical)
            .constraints([Constraint::Min(2)].as_ref())
            .split(fsize);
        let messages = state.messages.borrow();

        let spans: Vec<Spans> = messages
            .iter()
            .map(|msg| Spans::from(Span::styled(msg, Style::default().fg(Color::White))))
            .collect();

        let p = ratatui::widgets::Paragraph::new(spans)
            .wrap(Wrap { trim: true })
            .alignment(Alignment::Left)
            .block(
                Block::default()
                    .borders(Borders::ALL)
                    .style(Style::default().fg(Color::White))
                    .border_type(BorderType::Plain),
            );

        f.render_widget(p, chunks_main[0]);
    }
}
