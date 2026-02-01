use crate::config::Config;
use crate::data::{self, QuotaInfo, SessionInfo, StatsCache};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Panel {
    Summary,
    TokenUsage,
    CostsSummary,
    UsageQuota,
    CostBreakdown,
    ActivityChart,
    HourlyChart,
    Sessions,
}

impl Panel {
    pub fn next(self) -> Self {
        match self {
            Panel::Summary => Panel::TokenUsage,
            Panel::TokenUsage => Panel::CostsSummary,
            Panel::CostsSummary => Panel::UsageQuota,
            Panel::UsageQuota => Panel::CostBreakdown,
            Panel::CostBreakdown => Panel::ActivityChart,
            Panel::ActivityChart => Panel::HourlyChart,
            Panel::HourlyChart => Panel::Sessions,
            Panel::Sessions => Panel::Summary,
        }
    }

    pub fn prev(self) -> Self {
        match self {
            Panel::Summary => Panel::Sessions,
            Panel::TokenUsage => Panel::Summary,
            Panel::CostsSummary => Panel::TokenUsage,
            Panel::UsageQuota => Panel::CostsSummary,
            Panel::CostBreakdown => Panel::UsageQuota,
            Panel::ActivityChart => Panel::CostBreakdown,
            Panel::HourlyChart => Panel::ActivityChart,
            Panel::Sessions => Panel::HourlyChart,
        }
    }
}

pub struct App {
    pub config: Config,
    pub stats: Option<StatsCache>,
    pub sessions: Vec<SessionInfo>,
    pub quota: QuotaInfo,
    pub focused_panel: Panel,
    pub selected_session: usize,
    pub session_scroll: usize,
    pub is_live: bool,
    pub should_quit: bool,
    pub last_error: Option<String>,
}

impl App {
    pub fn new(config: Config) -> Self {
        Self {
            config,
            stats: None,
            sessions: Vec::new(),
            quota: QuotaInfo::default(),
            focused_panel: Panel::Summary,
            selected_session: 0,
            session_scroll: 0,
            is_live: true,
            should_quit: false,
            last_error: None,
        }
    }

    pub fn load_data(&mut self) {
        // Load stats
        match data::parse_stats_cache(&self.config.stats_file) {
            Ok(stats) => {
                self.stats = Some(stats);
                self.last_error = None;
            }
            Err(e) => {
                self.last_error = Some(format!("Stats: {}", e));
            }
        }

        // Load history
        match data::parse_history(&self.config.history_file) {
            Ok(entries) => {
                self.sessions = data::group_sessions(&entries, None);
            }
            Err(e) => {
                if self.last_error.is_none() {
                    self.last_error = Some(format!("History: {}", e));
                }
            }
        }
    }

    pub fn load_quota(&mut self) {
        match data::fetch_quota() {
            Ok(quota) => {
                self.quota = quota;
            }
            Err(e) => {
                self.quota.last_error = Some(e.to_string());
            }
        }
    }

    pub fn next_panel(&mut self) {
        self.focused_panel = self.focused_panel.next();
    }

    pub fn prev_panel(&mut self) {
        self.focused_panel = self.focused_panel.prev();
    }

    pub fn scroll_up(&mut self) {
        if self.focused_panel == Panel::Sessions && self.selected_session > 0 {
            self.selected_session -= 1;
            if self.selected_session < self.session_scroll {
                self.session_scroll = self.selected_session;
            }
        }
    }

    pub fn scroll_down(&mut self) {
        if self.focused_panel == Panel::Sessions && self.selected_session < self.sessions.len().saturating_sub(1) {
            self.selected_session += 1;
            // Adjust scroll to keep selection visible (assuming ~5 visible rows)
            if self.selected_session >= self.session_scroll + 4 {
                self.session_scroll = self.selected_session.saturating_sub(3);
            }
        }
    }

    pub fn refresh(&mut self) {
        self.load_data();
        self.load_quota();
    }

    pub fn quit(&mut self) {
        self.should_quit = true;
    }

    pub fn toggle_live(&mut self) {
        self.is_live = !self.is_live;
    }
}
