use std::time::SystemTime;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum UsageSource {
    Unknown,
    Remote,
    LocalSession,
}

impl UsageSource {
    pub fn label(self) -> &'static str {
        match self {
            Self::Unknown => "unknown",
            Self::Remote => "remote",
            Self::LocalSession => "local session",
        }
    }
}

impl Default for UsageSource {
    fn default() -> Self {
        Self::Unknown
    }
}

#[derive(Clone, Debug, Default)]
pub struct UsageSection {
    /// Kept as the bar fill percentage for the existing renderer.
    pub percentage: f64,
    pub used_percent: f64,
    pub remaining_percent: f64,
    pub resets_at: Option<SystemTime>,
    pub source: UsageSource,
    pub stale: bool,
}

impl UsageSection {
    pub fn from_used_percent(
        used_percent: f64,
        resets_at: Option<SystemTime>,
        source: UsageSource,
    ) -> Self {
        let used_percent = used_percent.clamp(0.0, 100.0);
        let remaining_percent = 100.0 - used_percent;
        let stale = resets_at.is_some_and(|reset| reset <= SystemTime::now());
        Self {
            percentage: used_percent,
            used_percent,
            remaining_percent,
            resets_at,
            source,
            stale,
        }
    }

    pub fn from_remaining_percent(
        remaining_percent: f64,
        resets_at: Option<SystemTime>,
        source: UsageSource,
    ) -> Self {
        let remaining_percent = remaining_percent.clamp(0.0, 100.0);
        Self::from_used_percent(100.0 - remaining_percent, resets_at, source)
    }
}

#[derive(Clone, Debug, Default)]
pub struct UsageData {
    pub session: UsageSection,
    pub weekly: UsageSection,
}

#[derive(Clone, Debug, Default)]
pub struct AppUsageData {
    pub claude_code: Option<UsageData>,
    pub codex: Option<UsageData>,
    pub antigravity: Option<UsageData>,
}
