#!/usr/bin/env python3
"""
Zero Human Daemon - Autonomous AI OS Development Loop

This daemon orchestrates the self-improving AI OS by:
1. Executing improvement scripts automatically
2. Running pixel programs on GPU via ai_runtime API
3. Saving results as cartridges
4. Learning from execution patterns
5. Generating new improvements autonomously
"""

import asyncio
import sqlite3
import time
import json
import sys
from pathlib import Path
from datetime import datetime
from typing import Optional, Dict, Any, List
import aiohttp

class ZeroHumanDaemon:
    """The autonomous orchestrator for the GPU AI OS"""

    def __init__(self, db_path: str = "db/daemon.db", api_url: str = "http://localhost:8081"):
        self.db_path = db_path
        self.api_url = api_url
        self.conn = None
        self.running = False
        self.cycle_count = 0

        # Ensure database directory exists
        Path(db_path).parent.mkdir(parents=True, exist_ok=True)

        # Initialize database
        self._init_database()

    def _init_database(self):
        """Initialize SQLite database with required tables"""
        self.conn = sqlite3.connect(self.db_path)
        self.conn.row_factory = sqlite3.Row

        # Create tables
        self.conn.executescript("""
            -- Improvement scripts (what to execute)
            CREATE TABLE IF NOT EXISTS improvement_scripts (
                id INTEGER PRIMARY KEY AUTOINCREMENT,
                name TEXT NOT NULL UNIQUE,
                lang TEXT NOT NULL CHECK(lang IN ('pixel', 'cartridge', 'python')),
                purpose TEXT,
                code TEXT NOT NULL,
                enabled INTEGER DEFAULT 1,
                created_at TEXT NOT NULL,
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

            -- Development goals (what to build)
            CREATE TABLE IF NOT EXISTS development_goals (
                id INTEGER PRIMARY KEY AUTOINCREMENT,
                goal TEXT NOT NULL,
                priority INTEGER DEFAULT 5,
                completed INTEGER DEFAULT 0,
                created_at TEXT NOT NULL,
                completed_at TEXT
            );

            -- Infinite map (history of everything built)
            CREATE TABLE IF NOT EXISTS infinite_map (
                id INTEGER PRIMARY KEY AUTOINCREMENT,
                iteration INTEGER NOT NULL,
                task_description TEXT,
                code TEXT,
                result TEXT,
                analysis TEXT,
                timestamp TEXT NOT NULL,
                parent_iteration INTEGER,
                FOREIGN KEY (parent_iteration) REFERENCES infinite_map(id)
            );

            -- Cartridge registry
            CREATE TABLE IF NOT EXISTS cartridge_registry (
                id INTEGER PRIMARY KEY AUTOINCREMENT,
                path TEXT NOT NULL UNIQUE,
                kind TEXT NOT NULL CHECK(kind IN ('pixel', 'code', 'dataset', 'prompt')),
                checksum TEXT,
                width INTEGER,
                height INTEGER,
                created_at TEXT NOT NULL,
                parent_id INTEGER,
                executed_count INTEGER DEFAULT 0,
                FOREIGN KEY (parent_id) REFERENCES cartridge_registry(id)
            );

            -- System metrics
            CREATE TABLE IF NOT EXISTS system_metrics (
                id INTEGER PRIMARY KEY AUTOINCREMENT,
                timestamp TEXT NOT NULL,
                metric_name TEXT NOT NULL,
                metric_value REAL NOT NULL,
                context TEXT
            );
        """)

        self.conn.commit()
        print("✅ Database initialized")

    async def check_api_health(self) -> bool:
        """Check if ai_runtime API is responding"""
        try:
            async with aiohttp.ClientSession() as session:
                async with session.get(f"{self.api_url}/health", timeout=aiohttp.ClientTimeout(total=2)) as resp:
                    if resp.status == 200:
                        text = await resp.text()
                        print(f"✅ API Health: {text}")
                        return True
                    return False
        except Exception as e:
            print(f"❌ API Health Check Failed: {e}")
            return False

    async def execute_pixel_program(self, source_code: str) -> Dict[str, Any]:
        """Execute a pixel program via the ai_runtime API"""

        # First, assemble the source code
        async with aiohttp.ClientSession() as session:
            # Step 1: Assemble
            async with session.post(
                f"{self.api_url}/api/pixel/assemble",
                json={"source": source_code}
            ) as resp:
                assemble_result = await resp.json()

                if not assemble_result.get("success"):
                    return {
                        "success": False,
                        "error": assemble_result.get("error", "Assembly failed"),
                        "stage": "assemble"
                    }

                program = assemble_result["program"]
                print(f"✅ Assembled {len(program)} instructions")

            # Step 2: Execute
            async with session.post(
                f"{self.api_url}/api/pixel/run",
                json={
                    "program": program,
                    "backend": "GPU",
                    "max_cycles": 1024,
                    "canvas_width": 128,
                    "canvas_height": 64
                }
            ) as resp:
                exec_result = await resp.json()
                return exec_result

    async def run_pending_improvements(self, max_runs: int = 5):
        """Execute pending improvement scripts"""

        cursor = self.conn.execute("""
            SELECT id, name, lang, code, purpose
            FROM improvement_scripts
            WHERE enabled = 1
            ORDER BY
                CASE WHEN last_run_at IS NULL THEN 0 ELSE 1 END,
                last_run_at ASC
            LIMIT ?
        """, (max_runs,))

        scripts = cursor.fetchall()

        if not scripts:
            print("📭 No pending improvement scripts")
            return

        print(f"\n{'='*60}")
        print(f"🚀 Executing {len(scripts)} improvement script(s)")
        print(f"{'='*60}\n")

        for script in scripts:
            await self._execute_script(dict(script))

    async def _execute_script(self, script: Dict[str, Any]):
        """Execute a single improvement script"""
        script_id = script['id']
        name = script['name']
        lang = script['lang']
        code = script['code']

        print(f"\n📝 Script: {name}")
        print(f"   Language: {lang}")
        print(f"   Purpose: {script.get('purpose', 'N/A')}")

        started_at = datetime.now().isoformat()
        start_time = time.time()

        # Record run start
        run_cursor = self.conn.execute("""
            INSERT INTO improvement_runs (script_id, started_at)
            VALUES (?, ?)
        """, (script_id, started_at))
        run_id = run_cursor.lastrowid
        self.conn.commit()

        try:
            result = None
            backend = "unknown"

            if lang == "pixel":
                # Execute pixel program
                result = await self.execute_pixel_program(code)
                backend = result.get("backend", "unknown")
                success = result.get("success", False)
                stdout = json.dumps(result, indent=2)
                stderr = result.get("error", "")

                if success:
                    print(f"   ✅ Execution succeeded ({result.get('cycles_executed', 0)} cycles)")
                else:
                    print(f"   ❌ Execution failed: {stderr}")

            elif lang == "cartridge":
                # Execute a cartridge by ID/path
                print(f"   📦 Executing cartridge: {code}")
                # TODO: Implement cartridge execution
                success = False
                stdout = "Cartridge execution not yet implemented"
                stderr = ""

            elif lang == "python":
                # Execute Python code (sandboxed)
                print(f"   🐍 Executing Python code")
                # TODO: Implement sandboxed Python execution
                success = False
                stdout = "Python execution not yet implemented"
                stderr = ""

            else:
                success = False
                stdout = ""
                stderr = f"Unsupported language: {lang}"

            # Record completion
            finished_at = datetime.now().isoformat()
            duration_ms = int((time.time() - start_time) * 1000)

            self.conn.execute("""
                UPDATE improvement_runs
                SET finished_at = ?, success = ?, stdout = ?, stderr = ?,
                    duration_ms = ?, backend = ?
                WHERE id = ?
            """, (finished_at, int(success), stdout, stderr, duration_ms, backend, run_id))

            # Update script statistics
            self.conn.execute("""
                UPDATE improvement_scripts
                SET last_run_at = ?,
                    run_count = run_count + 1,
                    success_count = success_count + ?
                WHERE id = ?
            """, (finished_at, int(success), script_id))

            self.conn.commit()

            # Save to infinite map
            self._save_to_infinite_map(
                iteration=self.cycle_count,
                task_description=f"{name}: {script.get('purpose', 'N/A')}",
                code=code,
                result=stdout,
                analysis=f"Success: {success}, Duration: {duration_ms}ms"
            )

            print(f"   ⏱️  Duration: {duration_ms}ms")

        except Exception as e:
            print(f"   ❌ Exception: {e}")

            # Record error
            self.conn.execute("""
                UPDATE improvement_runs
                SET finished_at = ?, success = 0, stderr = ?
                WHERE id = ?
            """, (datetime.now().isoformat(), str(e), run_id))
            self.conn.commit()

    def _save_to_infinite_map(self, iteration: int, task_description: str,
                             code: str, result: str, analysis: str):
        """Save execution to the infinite development map"""
        self.conn.execute("""
            INSERT INTO infinite_map
            (iteration, task_description, code, result, analysis, timestamp)
            VALUES (?, ?, ?, ?, ?, ?)
        """, (
            iteration,
            task_description,
            code,
            result,
            analysis,
            datetime.now().isoformat()
        ))
        self.conn.commit()

    async def autonomous_cycle(self):
        """Single cycle of autonomous development"""
        self.cycle_count += 1

        print(f"\n{'='*80}")
        print(f"🤖 AUTONOMOUS CYCLE #{self.cycle_count}")
        print(f"{'='*80}")

        # 1. Check API health
        if not await self.check_api_health():
            print("⚠️  AI Runtime API not available, skipping cycle")
            return

        # 2. Execute pending improvements
        await self.run_pending_improvements(max_runs=3)

        # 3. TODO: Consult local LLM for next improvement
        # 4. TODO: Generate new improvement script
        # 5. TODO: Analyze results and update goals

        # 6. Record metrics
        self.conn.execute("""
            INSERT INTO system_metrics (timestamp, metric_name, metric_value)
            VALUES (?, 'cycle_count', ?)
        """, (datetime.now().isoformat(), self.cycle_count))
        self.conn.commit()

    async def run_forever(self, cycle_interval: int = 10):
        """Run the autonomous development loop forever"""
        self.running = True

        print("""
╔════════════════════════════════════════════════════════════════╗
║        🤖 ZERO HUMAN DAEMON - AUTONOMOUS AI OS 🤖              ║
╚════════════════════════════════════════════════════════════════╝

Starting autonomous development loop...

The daemon will:
  1. Execute improvement scripts
  2. Run pixel programs on GPU
  3. Save results as cartridges
  4. Learn from execution patterns
  5. Generate new improvements

Press Ctrl+C to stop.
""")

        try:
            while self.running:
                try:
                    await self.autonomous_cycle()
                except Exception as e:
                    print(f"\n❌ Error in autonomous cycle: {e}")
                    import traceback
                    traceback.print_exc()

                print(f"\n⏸️  Sleeping for {cycle_interval}s before next cycle...")
                await asyncio.sleep(cycle_interval)

        except KeyboardInterrupt:
            print("\n\n🛑 Stopping daemon...")
            self.running = False
        finally:
            if self.conn:
                self.conn.close()
                print("✅ Database connection closed")

async def main():
    """Main entry point"""
    daemon = ZeroHumanDaemon()
    await daemon.run_forever(cycle_interval=10)

if __name__ == "__main__":
    asyncio.run(main())
