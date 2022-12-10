use std::{
    sync::mpsc::{channel, Receiver, RecvError, Sender},
    thread,
    time::Duration,
};

use crossterm::event::{self, KeyEvent, MouseButton};

pub enum InputEvent {
    /// An input event occurred.
    Input(KeyEvent),
    ///
    LeftClick(u16, u16),
    /// An tick event occurred.
    Tick,
}

pub struct Events {
    rx: Receiver<InputEvent>,
    // Need to be kept around to prevent disposing the sender side.
    _tx: Sender<InputEvent>,
}

impl Events {
    pub fn new(tick_rate: Duration) -> Events {
        let (tx, rx) = channel();

        let event_tx = tx.clone(); // the thread::spawn own event_tx

        thread::spawn(move || {
            loop {
                // poll for tick rate duration, if no event, sent tick event.
                if crossterm::event::poll(tick_rate).unwrap() {
                    match event::read().unwrap() {
                        event::Event::Key(key) => event_tx.send(InputEvent::Input(key)).unwrap(),
                        event::Event::Mouse(mouse_event) => {
                            if let event::MouseEventKind::Down(MouseButton::Left) = mouse_event.kind
                            {
                                event_tx
                                    .send(InputEvent::LeftClick(
                                        mouse_event.column,
                                        mouse_event.row,
                                    ))
                                    .unwrap();
                            }
                        }
                        _ => (),
                    };
                }
                event_tx.send(InputEvent::Tick).unwrap();
            }
        });

        Events { rx, _tx: tx }
    }

    /// Attempts to read an event.
    /// This function block the current thread.
    pub fn next(&self) -> Result<InputEvent, RecvError> {
        self.rx.recv()
    }
}
