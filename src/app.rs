use crate::config::Config;
use crate::data::{
    self, QuotaInfo, SessionInfo, StatsCache, UsageTracker, DepletionStatus,
    TrendData, CacheEfficiency, Averages, WebSearchStats, ProjectStats,
};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Panel {
    Summary,
    TokenUsage,
    Trends,
    CacheEfficiency,
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
            Panel::TokenUsage => Panel::Trends,
            Panel::Trends => Panel::CacheEfficiency,
            Panel::CacheEfficiency => Panel::CostsSummary,
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
            Panel::Trends => Panel::TokenUsage,
            Panel::CacheEfficiency => Panel::Trends,
            Panel::CostsSummary => Panel::CacheEfficiency,
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
    // Live data from history.jsonl
    pub today_messages_live: u64,
    pub recent_5h_messages: u64,
    pub stats_last_updated: Option<String>,
    // New enhanced statistics
    pub usage_tracker: UsageTracker,
    pub trend_data: TrendData,
    pub cache_efficiency: CacheEfficiency,
    pub averages: Averages,
    pub web_search_stats: WebSearchStats,
    pub top_projects: Vec<ProjectStats>,
    pub monthly_projection: f64,
    // History entries for trend calculations
    history_entries: Vec<data::HistoryEntry>,
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
            today_messages_live: 0,
            recent_5h_messages: 0,
            stats_last_updated: None,
            usage_tracker: UsageTracker::default(),
            trend_data: TrendData::default(),
            cache_efficiency: CacheEfficiency::default(),
            averages: Averages::default(),
            web_search_stats: WebSearchStats::default(),
            top_projects: Vec::new(),
            monthly_projection: 0.0,
            history_entries: Vec::new(),
        }
    }

    pub fn load_data(&mut self) {
        // Load stats and get file modification time
        match data::parse_stats_cache(&self.config.stats_file) {
            Ok(stats) => {
                // Calculate derived statistics
                self.cache_efficiency = CacheEfficiency::calculate(&stats);
                self.averages = Averages::calculate(&stats);
                self.web_search_stats = WebSearchStats::calculate(&stats);
                self.monthly_projection = data::calculate_monthly_projection(&stats);

                self.stats = Some(stats);
                self.last_error = None;
            }
            Err(e) => {
                self.last_error = Some(format!("Stats: {}", e));
            }
        }

        // Get stats file modification time
        if let Ok(metadata) = std::fs::metadata(&self.config.stats_file) {
            if let Ok(modified) = metadata.modified() {
                let datetime: chrono::DateTime<chrono::Local> = modified.into();
                self.stats_last_updated = Some(datetime.format("%b %d %H:%M").to_string());
            }
        }

        // Load history
        match data::parse_history(&self.config.history_file) {
            Ok(entries) => {
                // Count live messages
                self.today_messages_live = data::count_today_messages(&entries);
                self.recent_5h_messages = data::count_recent_messages(&entries, 5);
                self.sessions = data::group_sessions(&entries, None);

                // Top projects
                self.top_projects = data::count_projects(&entries, 5);

                // Calculate trends
                let daily_activity = self.stats.as_ref()
                    .map(|s| s.daily_activity.as_slice())
                    .unwrap_or(&[]);
                self.trend_data = TrendData::calculate(&entries, daily_activity);

                // Store entries for future use
                self.history_entries = entries;
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
            Ok(mut quota) => {
                // Add sample to usage tracker
                self.usage_tracker.add_sample(quota.session_usage, quota.week_usage);

                // Calculate projections if we have enough data
                if self.usage_tracker.has_enough_data() {
                    // Session rate
                    quota.session_rate_per_hour = self.usage_tracker.calculate_rate_per_hour(true);

                    // Week rate
                    quota.week_rate_per_hour = self.usage_tracker.calculate_rate_per_hour(false);

                    // Session depletion
                    if let (Some(usage), Some(resets_at)) = (quota.session_usage, &quota.session_resets_at) {
                        match self.usage_tracker.session_depletion_status(usage, resets_at) {
                            DepletionStatus::Depleting { hours_remaining } => {
                                quota.session_hours_to_depletion = Some(hours_remaining);
                                quota.session_is_safe = Some(false);
                            }
                            DepletionStatus::Safe { hours_until_reset } => {
                                quota.session_hours_to_depletion = Some(hours_until_reset);
                                quota.session_is_safe = Some(true);
                            }
                            _ => {}
                        }
                    }

                    // Week depletion
                    if let (Some(usage), Some(resets_at)) = (quota.week_usage, &quota.week_resets_at) {
                        match self.usage_tracker.week_depletion_status(usage, resets_at) {
                            DepletionStatus::Depleting { hours_remaining } => {
                                quota.week_hours_to_depletion = Some(hours_remaining);
                                quota.week_is_safe = Some(false);
                            }
                            DepletionStatus::Safe { hours_until_reset } => {
                                quota.week_hours_to_depletion = Some(hours_until_reset);
                                quota.week_is_safe = Some(true);
                            }
                            _ => {}
                        }
                    }
                }

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
