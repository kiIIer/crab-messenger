use crate::client::redux::actions::Action;
use crossbeam_channel::{unbounded, Receiver, Sender};
use crossterm::event;
use crossterm::event::Event::Key;
use crossterm::event::{Event, KeyEventKind};
use std::thread;
use std::time::Duration;

static POLL_DURATION: Duration = Duration::from_millis(100);

pub struct Input {
    pub receiver: Receiver<Event>,
}

impl Input {
    pub fn new() -> Input {
        let (tx, rx) = unbounded();

        thread::spawn(move || {
            Self::input_loop(&tx);
        });

        Input { receiver: rx }
    }

    fn poll(dur: Duration) -> anyhow::Result<Option<Event>> {
        if event::poll(dur)? {
            Ok(Some(event::read()?))
        } else {
            Ok(None)
        }
    }

    fn input_loop(tx: &Sender<Event>) {
        loop {
            if let Some(e) = Self::poll(POLL_DURATION).unwrap() {
                if let Key(key) = e {
                    if key.kind != KeyEventKind::Press {
                        continue;
                    }
                }

                tx.send(e);
            }
        }
    }
}
