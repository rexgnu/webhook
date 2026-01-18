use crate::request::CapturedRequest;
use crossterm::event::{self, Event, KeyCode, KeyEventKind};
use std::time::Duration;
use tokio::sync::mpsc;

pub enum InputEvent {
    Key(KeyCode),
    NewRequest(CapturedRequest),
    Tick,
}

pub struct App {
    pub requests: Vec<CapturedRequest>,
    pub selected_index: usize,
    pub scroll_offset: usize,
    pub detail_scroll: usize,
    pub should_quit: bool,
    pub listening_address: String,
    pub body_expanded: bool,
}

impl App {
    pub fn new(listening_address: String) -> Self {
        Self {
            requests: Vec::new(),
            selected_index: 0,
            scroll_offset: 0,
            detail_scroll: 0,
            should_quit: false,
            listening_address,
            body_expanded: false,
        }
    }

    pub fn add_request(&mut self, request: CapturedRequest) {
        // Add to the beginning (newest first)
        self.requests.insert(0, request);
        // Keep selection valid
        if self.selected_index > 0 {
            self.selected_index += 1;
        }
    }

    pub fn selected_request(&self) -> Option<&CapturedRequest> {
        self.requests.get(self.selected_index)
    }

    pub fn move_selection_up(&mut self) {
        if self.selected_index > 0 {
            self.selected_index -= 1;
            self.detail_scroll = 0;
        }
    }

    pub fn move_selection_down(&mut self) {
        if !self.requests.is_empty() && self.selected_index < self.requests.len() - 1 {
            self.selected_index += 1;
            self.detail_scroll = 0;
        }
    }

    pub fn scroll_detail_up(&mut self) {
        if self.detail_scroll > 0 {
            self.detail_scroll -= 1;
        }
    }

    pub fn scroll_detail_down(&mut self) {
        self.detail_scroll += 1;
    }

    pub fn clear_requests(&mut self) {
        self.requests.clear();
        self.selected_index = 0;
        self.scroll_offset = 0;
        self.detail_scroll = 0;
    }

    pub fn toggle_body_expanded(&mut self) {
        self.body_expanded = !self.body_expanded;
    }

    pub fn handle_input(&mut self, event: InputEvent) {
        match event {
            InputEvent::Key(key) => match key {
                KeyCode::Char('q') | KeyCode::Esc => self.should_quit = true,
                KeyCode::Char('c') => self.clear_requests(),
                KeyCode::Up | KeyCode::Char('k') => self.move_selection_up(),
                KeyCode::Down | KeyCode::Char('j') => self.move_selection_down(),
                KeyCode::Enter => self.toggle_body_expanded(),
                KeyCode::PageUp => {
                    for _ in 0..5 {
                        self.scroll_detail_up();
                    }
                }
                KeyCode::PageDown => {
                    for _ in 0..5 {
                        self.scroll_detail_down();
                    }
                }
                _ => {}
            },
            InputEvent::NewRequest(req) => self.add_request(req),
            InputEvent::Tick => {}
        }
    }
}

pub async fn poll_events(rx: &mut mpsc::UnboundedReceiver<CapturedRequest>) -> Option<InputEvent> {
    // Check for new requests first (non-blocking)
    if let Ok(request) = rx.try_recv() {
        return Some(InputEvent::NewRequest(request));
    }

    // Poll for keyboard events with timeout
    if event::poll(Duration::from_millis(100)).ok()? {
        if let Ok(Event::Key(key)) = event::read() {
            if key.kind == KeyEventKind::Press {
                return Some(InputEvent::Key(key.code));
            }
        }
    }

    Some(InputEvent::Tick)
}
