use crate::config::Config;
use crate::data::{
    self, Averages, CacheEfficiency, DepletionStatus, ProjectStats, QuotaInfo, SessionInfo,
    StatsCache, TrendData, UsageTracker, WebSearchStats,
};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum AppView {
    Dashboard,
    Projects,
    Models,
}

impl AppView {
    pub fn next(self) -> Self {
        match self {
            Self::Dashboard => Self::Projects,
            Self::Projects => Self::Models,
            Self::Models => Self::Dashboard,
        }
    }

    pub fn prev(self) -> Self {
        match self {
            Self::Dashboard => Self::Models,
            Self::Projects => Self::Dashboard,
            Self::Models => Self::Projects,
        }
    }
}


#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ProjectSort {
    Cost,
    Recent,
}

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

const ALL_PANELS: &[Panel] = &[
    Panel::Summary,
    Panel::TokenUsage,
    Panel::Trends,
    Panel::CacheEfficiency,
    Panel::CostsSummary,
    Panel::UsageQuota,
    Panel::CostBreakdown,
    Panel::ActivityChart,
    Panel::HourlyChart,
    Panel::Sessions,
];

impl Panel {
    pub fn next(self) -> Self {
        let idx = ALL_PANELS.iter().position(|&p| p == self).unwrap_or(0);
        ALL_PANELS[(idx + 1) % ALL_PANELS.len()]
    }

    pub fn prev(self) -> Self {
        let idx = ALL_PANELS.iter().position(|&p| p == self).unwrap_or(0);
        ALL_PANELS[(idx + ALL_PANELS.len() - 1) % ALL_PANELS.len()]
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
    pub export_message: Option<String>,
    pub selected_session_id: Option<String>,
    pub session_details_scroll: usize,
    pub current_view: AppView,
    pub project_sort: ProjectSort,
    pub is_filtering_projects: bool,
    pub project_search_query: String,

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
    pub total_cost: f64,
    pub monthly_projection: f64,
    // History entries for trend calculations
    history_entries: Vec<data::HistoryEntry>,
}

impl App {
    pub fn new(config: Config) -> Self {
        // Try to load existing samples from disk
        let usage_tracker = UsageTracker::load_from_file(&config.samples_file).unwrap_or_default();

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
            export_message: None,
            selected_session_id: None,
            session_details_scroll: 0,
            current_view: AppView::Dashboard,
            project_sort: ProjectSort::Cost,
            is_filtering_projects: false,
            project_search_query: String::new(),

            today_messages_live: 0,
            recent_5h_messages: 0,
            stats_last_updated: None,
            usage_tracker,
            trend_data: TrendData::default(),
            cache_efficiency: CacheEfficiency::default(),
            averages: Averages::default(),
            web_search_stats: WebSearchStats::default(),
            top_projects: Vec::new(),
            total_cost: 0.0,
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
                self.total_cost = data::calculate_total_cost(&stats);
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
            Ok(recent_entries) => {
                // Sync with archive
                let entries = data::sync_history_archive(&self.config.archive_file, &recent_entries)
                    .unwrap_or(recent_entries);

                // Count live messages
                self.today_messages_live = data::count_today_messages(&entries);
                self.recent_5h_messages = data::count_recent_messages(&entries, 5);
                self.sessions = data::group_sessions(&entries, None);

                let total_global_tokens = self.stats.as_ref().map(|s| {
                    s.model_usage.values().map(|u| u.input_tokens + u.output_tokens + u.cache_read_input_tokens + u.cache_creation_input_tokens).sum::<u64>()
                }).unwrap_or(0);
                let total_global_cost = self.total_cost;

                // Load all projects (0 means no truncate) for the Project Costs tab
                self.top_projects = data::count_projects(&entries, 0, total_global_tokens, total_global_cost);


                // Calculate trends
                let daily_activity = self
                    .stats
                    .as_ref()
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

    /// Initial synchronous quota load (used at startup)
    pub fn load_quota(&mut self) {
        match data::fetch_quota() {
            Ok(quota) => self.process_quota(quota),
            Err(e) => {
                let err_msg = e.to_string();
                if err_msg != "DEBOUNCED" {
                    self.quota.last_error = Some(err_msg);
                }
            }
        }
    }

    /// Handle async quota result from background thread
    pub fn apply_quota_result(&mut self, result: Result<data::QuotaInfo, String>) {
        match result {
            Ok(quota) => self.process_quota(quota),
            Err(e) => {
                if e != "DEBOUNCED" {
                    self.quota.last_error = Some(e);
                }
            }
        }
    }

    /// Process a successfully fetched QuotaInfo (shared by sync and async paths)
    fn process_quota(&mut self, mut quota: data::QuotaInfo) {
        // Add sample to usage tracker
        self.usage_tracker
            .add_sample(quota.session_usage, quota.week_usage);

        // Save samples periodically (every 3 samples = ~15 minutes at 5m intervals)
        if self.usage_tracker.sample_count() % 3 == 0 {
            let _ = self.usage_tracker.save_to_file(&self.config.samples_file);
        }

        // Calculate projections if we have enough data
        if self.usage_tracker.has_enough_data() {
            // Session rate
            quota.session_rate_per_hour = self.usage_tracker.calculate_rate_per_hour(true);

            // Week rate
            quota.week_rate_per_hour = self.usage_tracker.calculate_rate_per_hour(false);

            // Session depletion
            if let (Some(usage), Some(resets_at)) = (quota.session_usage, &quota.session_resets_at)
            {
                match self
                    .usage_tracker
                    .session_depletion_status(usage, resets_at)
                {
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

    pub fn next_panel(&mut self) {
        self.focused_panel = self.focused_panel.next();
    }

    pub fn prev_panel(&mut self) {
        self.focused_panel = self.focused_panel.prev();
    }

    pub fn scroll_up(&mut self) {
        if self.selected_session_id.is_some() {
            if self.session_details_scroll > 0 {
                self.session_details_scroll -= 1;
            }
        } else if self.focused_panel == Panel::Sessions && self.selected_session > 0 {
            self.selected_session -= 1;
            if self.selected_session < self.session_scroll {
                self.session_scroll = self.selected_session;
            }
        }
    }

    pub fn scroll_down(&mut self) {
        if self.selected_session_id.is_some() {
            // Unbounded scroll for now, could bound by number of messages
            self.session_details_scroll += 1;
        } else if self.focused_panel == Panel::Sessions
            && self.selected_session < self.sessions.len().saturating_sub(1)
        {
            self.selected_session += 1;
            // Adjust scroll to keep selection visible (assuming ~5 visible rows)
            if self.selected_session >= self.session_scroll + 4 {
                self.session_scroll = self.selected_session.saturating_sub(3);
            }
        }
    }

    pub fn quit(&mut self) {
        // Save usage tracker samples before exiting
        let _ = self.usage_tracker.save_to_file(&self.config.samples_file);
        self.should_quit = true;
    }

    pub fn toggle_live(&mut self) {
        self.is_live = !self.is_live;
    }

    pub fn toggle_model_breakdown(&mut self) {
        if self.current_view == AppView::Models {
            self.current_view = AppView::Dashboard;
        } else {
            self.current_view = AppView::Models;
            self.selected_session_id = None;
        }
    }

    pub fn toggle_project_costs(&mut self) {
        if self.current_view == AppView::Projects {
            self.current_view = AppView::Dashboard;
        } else {
            self.current_view = AppView::Projects;
            self.selected_session_id = None;
        }
    }

    pub fn next_view(&mut self) {
        self.current_view = self.current_view.next();
        self.selected_session_id = None;
    }

    pub fn prev_view(&mut self) {
        self.current_view = self.current_view.prev();
        self.selected_session_id = None;
    }


    pub fn toggle_project_sort(&mut self) {
        self.project_sort = match self.project_sort {
            ProjectSort::Cost => ProjectSort::Recent,
            ProjectSort::Recent => ProjectSort::Cost,
        };
    }

    pub fn export_data(&mut self) {
        use serde::Serialize;

        #[derive(Serialize)]
        struct ExportData<'a> {
            stats: Option<&'a StatsCache>,
            top_projects: &'a [ProjectStats],
            total_cost: f64,
            monthly_projection: f64,
            timestamp: chrono::DateTime<chrono::Local>,
        }

        let data = ExportData {
            stats: self.stats.as_ref(),
            top_projects: &self.top_projects,
            total_cost: self.total_cost,
            monthly_projection: self.monthly_projection,
            timestamp: chrono::Local::now(),
        };

        let home = match dirs::home_dir() {
            Some(path) => path,
            None => {
                self.export_message = Some("Failed to find home directory".to_string());
                return;
            }
        };

        let export_path = home.join("claude_monitor_export.json");

        match serde_json::to_string_pretty(&data) {
            Ok(json) => match std::fs::write(&export_path, json) {
                Ok(_) => {
                    self.export_message = Some(format!("Exported to {}", export_path.display()));
                }
                Err(e) => {
                    self.export_message = Some(format!("Export failed: {}", e));
                }
            },
            Err(e) => {
                self.export_message = Some(format!("Serialization failed: {}", e));
            }
        }
    }

    pub fn cleanup_ghost_sessions(&mut self) {
        let current_session_file = self.config.claude_dir.join("current_session_id");
        if !current_session_file.exists() {
            self.export_message = Some("No active session file found.".to_string());
            return;
        }

        match std::fs::remove_file(&current_session_file) {
            Ok(_) => {
                self.export_message = Some("Cleared ghost session correctly.".to_string());
                self.load_data();
            }
            Err(e) => {
                self.last_error = Some(format!("Failed to clear session: {}", e));
            }
        }
    }

    pub fn select_current_session(&mut self) {
        if self.focused_panel == Panel::Sessions && self.selected_session < self.sessions.len() {
            let session = &self.sessions[self.selected_session];
            self.selected_session_id = Some(session.session_id.clone());
            self.session_details_scroll = 0;
        }
    }

    pub fn close_session_details(&mut self) -> bool {
        if self.selected_session_id.is_some() {
            self.selected_session_id = None;
            true
        } else {
            false
        }
    }

    pub fn get_session_history(&self, session_id: &str) -> Vec<&data::HistoryEntry> {
        let mut entries: Vec<_> = self
            .history_entries
            .iter()
            .filter(|e| e.session_id == session_id)
            .collect();
        entries.sort_by_key(|e| e.timestamp);
        entries
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn test_app() -> App {
        let config = Config::default();
        let mut app = App::new(config);
        // Add some mock sessions for scroll tests
        app.sessions = (0..10)
            .map(|i| data::SessionInfo {
                session_id: format!("session-{}", i),
                project: format!("/test/project-{}", i),
                project_name: format!("project-{}", i),
                first_timestamp: 1000 * i as u64,
                last_timestamp: 2000 * i as u64,
                message_count: (i + 1) as u64,
                is_active: i == 0,
            })
            .collect();
        app
    }

    #[test]
    fn test_panel_next_cycles_through_all() {
        let mut panel = Panel::Summary;
        let mut visited = vec![panel];
        for _ in 0..(ALL_PANELS.len() - 1) {
            panel = panel.next();
            visited.push(panel);
        }
        assert_eq!(visited.len(), ALL_PANELS.len());
        // Full cycle returns to start
        assert_eq!(panel.next(), Panel::Summary);
    }

    #[test]
    fn test_panel_prev_cycles_through_all() {
        let mut panel = Panel::Summary;
        let mut visited = vec![panel];
        for _ in 0..(ALL_PANELS.len() - 1) {
            panel = panel.prev();
            visited.push(panel);
        }
        assert_eq!(visited.len(), ALL_PANELS.len());
        // Full cycle returns to start
        assert_eq!(panel.prev(), Panel::Summary);
    }

    #[test]
    fn test_panel_next_prev_are_inverse() {
        for &panel in ALL_PANELS {
            assert_eq!(panel.next().prev(), panel);
            assert_eq!(panel.prev().next(), panel);
        }
    }

    #[test]
    fn test_scroll_down_increments() {
        let mut app = test_app();
        app.focused_panel = Panel::Sessions;
        assert_eq!(app.selected_session, 0);
        app.scroll_down();
        assert_eq!(app.selected_session, 1);
        app.scroll_down();
        assert_eq!(app.selected_session, 2);
    }

    #[test]
    fn test_scroll_up_at_zero_stays() {
        let mut app = test_app();
        app.focused_panel = Panel::Sessions;
        assert_eq!(app.selected_session, 0);
        app.scroll_up();
        assert_eq!(app.selected_session, 0);
    }

    #[test]
    fn test_scroll_down_stops_at_end() {
        let mut app = test_app();
        app.focused_panel = Panel::Sessions;
        for _ in 0..20 {
            app.scroll_down();
        }
        assert_eq!(app.selected_session, 9); // 10 sessions, 0-indexed
    }

    #[test]
    fn test_scroll_only_in_sessions_panel() {
        let mut app = test_app();
        app.focused_panel = Panel::Summary;
        app.scroll_down();
        assert_eq!(app.selected_session, 0); // No change
    }

    #[test]
    fn test_toggle_live() {
        let mut app = test_app();
        assert!(app.is_live);
        app.toggle_live();
        assert!(!app.is_live);
        app.toggle_live();
        assert!(app.is_live);
    }

    #[test]
    fn test_apply_quota_result_success() {
        let mut app = test_app();
        let quota = data::QuotaInfo {
            session_usage: Some(42.0),
            week_usage: Some(15.0),
            ..Default::default()
        };
        app.apply_quota_result(Ok(quota));
        assert_eq!(app.quota.session_usage, Some(42.0));
        assert_eq!(app.quota.week_usage, Some(15.0));
        assert!(app.quota.last_error.is_none());
    }

    #[test]
    fn test_apply_quota_result_error() {
        let mut app = test_app();
        app.apply_quota_result(Err("API timeout".to_string()));
        assert_eq!(app.quota.last_error, Some("API timeout".to_string()));
    }
}
