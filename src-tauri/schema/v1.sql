-- Schema version tracking
CREATE TABLE IF NOT EXISTS schema_versions (
    version INTEGER PRIMARY KEY,
    applied_at DATETIME DEFAULT CURRENT_TIMESTAMP
);

-- Main settings table
CREATE TABLE IF NOT EXISTS settings (
    id INTEGER PRIMARY KEY CHECK (id = 1),
    theme TEXT DEFAULT 'dark',
    opacity REAL DEFAULT 0.85,
    font_size INTEGER DEFAULT 14,
    always_on_top INTEGER DEFAULT 1,
    auto_start INTEGER DEFAULT 0,
    notifications INTEGER DEFAULT 1,
    sound INTEGER DEFAULT 1,
    sound_volume REAL DEFAULT 0.7,
    http_port INTEGER DEFAULT 31415,
    custom_colors TEXT DEFAULT '{}',
    reminder_threshold INTEGER DEFAULT 100,
    do_not_disturb INTEGER DEFAULT 0,
    do_not_disturb_start TEXT DEFAULT '22:00',
    do_not_disturb_end TEXT DEFAULT '08:00',
    window_visible INTEGER DEFAULT 1,
    language TEXT DEFAULT 'en',
    updated_at DATETIME DEFAULT CURRENT_TIMESTAMP
);

-- Task history table for enhanced task management
CREATE TABLE IF NOT EXISTS task_history (
    id TEXT PRIMARY KEY,
    name TEXT NOT NULL,
    status TEXT,
    progress INTEGER DEFAULT 0,
    tokens INTEGER DEFAULT 0,
    ide TEXT,
    window_title TEXT,
    project_path TEXT,
    start_time INTEGER,
    end_time INTEGER,
    created_at DATETIME DEFAULT CURRENT_TIMESTAMP
);

-- Index for quick lookups
CREATE INDEX IF NOT EXISTS idx_task_history_status ON task_history(status);
CREATE INDEX IF NOT EXISTS idx_task_history_created ON task_history(created_at DESC);

-- Insert initial settings row if not exists
INSERT OR IGNORE INTO settings (id) VALUES (1);

-- Record schema version
INSERT OR IGNORE INTO schema_versions (version) VALUES (1);
