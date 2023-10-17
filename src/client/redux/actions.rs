use crossterm::event::Event;

pub enum Action {
    Input(Event),
}
