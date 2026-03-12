use super::log_level::LogLevel;

pub(crate) struct LevelFilters {
    trace: bool,
    debug: bool,
    info: bool,
    warn: bool,
    error: bool,
}

impl LevelFilters {
    pub(crate) fn new() -> Self {
        Self {
            trace: false,
            debug: false,
            info: false,
            warn: false,
            error: false,
        }
    }

    pub(crate) fn matches(&self, level: LogLevel) -> bool {
        if !self.any_active() {
            return true;
        }

        match level {
            LogLevel::Trace => self.trace,
            LogLevel::Debug => self.debug,
            LogLevel::Info => self.info,
            LogLevel::Warn => self.warn,
            LogLevel::Error => self.error,
            LogLevel::Unknown => false,
        }
    }

    pub(crate) fn toggle(&mut self, level: LogLevel) {
        match level {
            LogLevel::Trace => self.trace = !self.trace,
            LogLevel::Debug => self.debug = !self.debug,
            LogLevel::Info => self.info = !self.info,
            LogLevel::Warn => self.warn = !self.warn,
            LogLevel::Error => self.error = !self.error,
            LogLevel::Unknown => {}
        }
    }

    pub(crate) fn set_active_levels(&mut self, levels: &[LogLevel]) {
        self.clear();
        for level in levels {
            self.toggle_on(*level);
        }
    }

    pub(crate) fn clear(&mut self) {
        self.trace = false;
        self.debug = false;
        self.info = false;
        self.warn = false;
        self.error = false;
    }

    pub(crate) fn summary(&self) -> String {
        if !self.any_active() {
            return "all".to_string();
        }

        let mut levels = Vec::new();
        if self.trace {
            levels.push("trace");
        }
        if self.debug {
            levels.push("debug");
        }
        if self.info {
            levels.push("info");
        }
        if self.warn {
            levels.push("warn");
        }
        if self.error {
            levels.push("error");
        }

        levels.join(",")
    }

    fn any_active(&self) -> bool {
        self.trace || self.debug || self.info || self.warn || self.error
    }

    fn toggle_on(&mut self, level: LogLevel) {
        match level {
            LogLevel::Trace => self.trace = true,
            LogLevel::Debug => self.debug = true,
            LogLevel::Info => self.info = true,
            LogLevel::Warn => self.warn = true,
            LogLevel::Error => self.error = true,
            LogLevel::Unknown => {}
        }
    }
}
