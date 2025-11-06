-- Database schema for Zero Human Daemon
-- File-socket IPC version using GVPIE

-- Improvement scripts (what to execute)
CREATE TABLE IF NOT EXISTS improvement_scripts (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    name TEXT NOT NULL UNIQUE,
    lang TEXT NOT NULL CHECK(lang IN ('pixel', 'gvpie', 'cartridge', 'python')),
    code TEXT NOT NULL,
    enabled INTEGER DEFAULT 1,
    created_at TEXT DEFAULT (datetime('now')),
    last_run_at TEXT,
    run_count INTEGER DEFAULT 0,
    success_count INTEGER DEFAULT 0
);

-- Improvement runs (execution history)
CREATE TABLE IF NOT EXISTS improvement_runs (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    script_id INTEGER NOT NULL,
    started_at TEXT NOT NULL,
    finished_at TEXT,
    success INTEGER,
    stdout TEXT,
    stderr TEXT,
    duration_ms INTEGER,
    backend TEXT,
    cartridge_path TEXT,
    FOREIGN KEY (script_id) REFERENCES improvement_scripts(id)
);

-- First improvement script: hello_gpu
INSERT INTO improvement_scripts (name, lang, code, enabled)
VALUES ('hello_gpu', 'gvpie', 'TXT 10 10 Hello GPU
HALT', 1);
