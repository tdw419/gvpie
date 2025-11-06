#!/usr/bin/env python3
"""Create the daemon database from schema file."""

import sqlite3
from pathlib import Path

DB_PATH = Path(__file__).parent / "db" / "daemon.db"
SCHEMA_PATH = Path(__file__).parent / "schema_autonomous.sql"

# Ensure db directory exists
DB_PATH.parent.mkdir(parents=True, exist_ok=True)

# Create database
conn = sqlite3.connect(str(DB_PATH))

# Read and execute schema
schema = SCHEMA_PATH.read_text()
conn.executescript(schema)
conn.commit()
conn.close()

print(f"✅ Database created successfully at: {DB_PATH}")
print(f"📊 Schema loaded from: {SCHEMA_PATH}")
