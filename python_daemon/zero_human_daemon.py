#!/usr/bin/env python3
"""
Zero Human Daemon - Autonomous GPU AI OS

Polls improvement_scripts table, executes pixel programs via GVPIE file-socket,
and saves cartridges to disk.
"""

import sqlite3
import time
from pathlib import Path
from datetime import datetime

from pixel_os.pixel_runner import PixelRunner

# Paths
DB_PATH = Path(__file__).parent / "db" / "daemon.db"
CARTRIDGE_DIR = Path(__file__).parent / "cartridges"

class ZHIDaemon:
    """Zero Human Interface Daemon - autonomous orchestrator."""

    def __init__(self, db_path: Path = DB_PATH):
        self.db_path = db_path
        self.runner = PixelRunner(timeout=2.0)
        self.conn = None

        # Ensure directories exist
        db_path.parent.mkdir(parents=True, exist_ok=True)
        CARTRIDGE_DIR.mkdir(parents=True, exist_ok=True)

        # Initialize database
        self._init_db()

    def _init_db(self):
        """Connect to SQLite database."""
        self.conn = sqlite3.connect(str(self.db_path))
        self.conn.row_factory = sqlite3.Row
        print(f"✅ Connected to database: {self.db_path}")

    def run_once(self):
        """Execute one pending improvement script."""
        cur = self.conn.execute("""
            SELECT id, name, lang, code
            FROM improvement_scripts
            WHERE enabled = 1
            ORDER BY id ASC
            LIMIT 1
        """)

        row = cur.fetchone()
        if not row:
            print("📭 No pending improvement scripts")
            return

        script_id = row["id"]
        name = row["name"]
        lang = row["lang"]
        code = row["code"]

        print(f"\n{'='*60}")
        print(f"🚀 Executing: {name} (lang={lang})")
        print(f"{'='*60}")

        started = time.time()

        try:
            if lang in ("gvpie", "pixel"):
                # Execute pixel program via GVPIE file-socket
                img = self.runner.execute_text_program(code)

                # Save cartridge
                out_path = CARTRIDGE_DIR / f"{name}_{int(started)}.png"
                self.runner.save_cartridge(img, out_path)

                success = True
                stdout = f"Saved cartridge: {out_path}"
                stderr = ""

                print(f"✅ Success! Saved to: {out_path}")

            else:
                success = False
                stdout = ""
                stderr = f"Unsupported lang: {lang}"
                print(f"❌ {stderr}")

        except Exception as e:
            success = False
            stdout = ""
            stderr = str(e)
            print(f"❌ Exception: {e}")

        finished = time.time()
        duration_ms = int((finished - started) * 1000)

        # Record run in database
        self.conn.execute("""
            INSERT INTO improvement_runs
            (script_id, started_at, finished_at, success, stdout, stderr, duration_ms)
            VALUES (?, ?, ?, ?, ?, ?, ?)
        """, (
            script_id,
            datetime.fromtimestamp(started).isoformat(),
            datetime.fromtimestamp(finished).isoformat(),
            int(success),
            stdout,
            stderr,
            duration_ms,
        ))

        # Update script stats
        self.conn.execute("""
            UPDATE improvement_scripts
            SET
                last_run_at = ?,
                run_count = run_count + 1,
                success_count = success_count + ?
            WHERE id = ?
        """, (
            datetime.fromtimestamp(finished).isoformat(),
            int(success),
            script_id,
        ))

        self.conn.commit()
        print(f"⏱️  Duration: {duration_ms}ms\n")

    def run_forever(self, interval: float = 10.0):
        """Run autonomous loop forever."""
        print("""
╔════════════════════════════════════════════════════════════════╗
║        🤖 ZERO HUMAN DAEMON - AUTONOMOUS AI OS 🤖              ║
╚════════════════════════════════════════════════════════════════╝

Starting autonomous development loop...
Press Ctrl+C to stop.
""")

        try:
            while True:
                self.run_once()
                print(f"⏸️  Sleeping for {interval}s before next cycle...")
                time.sleep(interval)
        except KeyboardInterrupt:
            print("\n🛑 Stopping daemon...")
        finally:
            if self.conn:
                self.conn.close()
                print("✅ Database connection closed")

def main():
    """Main entry point."""
    daemon = ZHIDaemon()
    daemon.run_forever(interval=10.0)

if __name__ == "__main__":
    main()
