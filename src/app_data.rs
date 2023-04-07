use std::collections::VecDeque;
use crate::consts::*;
#[derive(Default)]
pub(crate) struct AppData {
    pub data: Vec<(f64, f64)>,
    pub screen_refresh_secs: u32,
    pub tick_interval_msecs: u64,
    pub log_buffer: VecDeque<String>
}
impl AppData {
    pub(crate) fn default() -> AppData {
        AppData {
            data: vec![],
            screen_refresh_secs: DEFAULT_SCREEN_REFRESH_RATE_SECS,
            tick_interval_msecs: DEFAULT_TICK_INTERVAL_MSECS,
            log_buffer: VecDeque::with_capacity(DEFAULT_LOG_RING_BUFFER_SIZE)
        }
    }

    pub(crate) fn on_tick(&mut self) {
        debug!("tick");
    }
}
