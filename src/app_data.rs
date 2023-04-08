use std::borrow::Cow;
use std::collections::VecDeque;
use std::sync::{Arc, Mutex};
use crate::consts::*;
use tui_menu::{MenuItem, MenuState};

pub(crate) struct AppData {
    pub data: Vec<(f64, f64)>,
    pub screen_refresh_secs: u32,
    pub tick_interval_msecs: u64,
    pub log_buffer: Arc<Mutex<VecDeque<String>>>,
    pub menu: MenuState<Cow<'static, str>>
}
impl AppData {
    pub(crate) fn default() -> AppData {
        AppData {
            data: vec![],
            screen_refresh_secs: DEFAULT_SCREEN_REFRESH_RATE_SECS,
            tick_interval_msecs: DEFAULT_TICK_INTERVAL_MSECS,
            log_buffer: Arc::new(Mutex::new(VecDeque::with_capacity(DEFAULT_LOG_RING_BUFFER_SIZE))),
            menu: make_menu()
        }
    }

    pub(crate) fn on_tick(&mut self) {
        debug!("tick");
    }
}
fn make_menu()-> MenuState<Cow<'static, str>> {
    MenuState::new(vec![
        MenuItem::group(
            "File",
            vec![MenuItem::item("Exit", "exit".into())]
        ),
        MenuItem::group(
            "Help",
    vec![MenuItem::item("About", "about".into())]),

    ])
}