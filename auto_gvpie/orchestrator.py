"""
Auto-GVPIE: Autonomous development loop
Drives GVPIE development using local LLM + LanceDB + continuous feedback

Flow:
1. Pick next task from queue
2. Ask local LLM (LM Studio) for implementation
3. Apply patch and test
4. Store results in LanceDB
5. If error, feed back to LLM for fix
6. If success, move to next task
7. Output pixel representations
"""

import json
import requests
import subprocess
import uuid
import datetime
import pathlib
from typing import Optional, Dict, Any, List
from dataclasses import dataclass, asdict

# Configuration
LM_STUDIO_URL = "http://127.0.0.1:1234/v1/chat/completions"
REPO_ROOT = pathlib.Path(__file__).parent.parent
TASKS_FILE = REPO_ROOT / "auto_tasks.json"
ARTIFACTS_DIR = REPO_ROOT / "artifacts"
ARTIFACTS_DIR.mkdir(exist_ok=True)

# Try to import lancedb, but make it optional
try:
    import lancedb
    LANCEDB_AVAILABLE = True
    DB_PATH = str(REPO_ROOT / "lancedb")
except ImportError:
    LANCEDB_AVAILABLE = False
    print("⚠️  LanceDB not available, using file-based storage")


@dataclass
class Task:
    """A development task for the LLM to complete"""
    id: str
    title: str
    inputs: List[str]  # Files to consider
    tests: List[str]   # Commands to run for validation
    status: str  # pending, in_progress, done, failed
    priority: int = 5
    created_at: str = ""

    def __post_init__(self):
        if not self.created_at:
            self.created_at = datetime.datetime.utcnow().isoformat()


@dataclass
class LLMArtifact:
    """Output from LLM: spec, patch, or pixel program"""
    id: str
    task_id: str
    kind: str  # "spec", "patch", "pixel_program"
    content: Dict[str, Any]
    embedding_text: str
    tags: List[str]
    created_at: str = ""

    def __post_init__(self):
        if not self.created_at:
            self.created_at = datetime.datetime.utcnow().isoformat()


@dataclass
class BuildRun:
    """Result of building/testing an artifact"""
    id: str
    task_id: str
    artifact_id: str
    command: str
    exit_code: int
    stdout: str
    stderr: str
    pixel_path: Optional[str] = None
    created_at: str = ""

    def __post_init__(self):
        if not self.created_at:
            self.created_at = datetime.datetime.utcnow().isoformat()


class LLMInterface:
    """Interface to local LLM (LM Studio)"""

    def __init__(self, url: str = LM_STUDIO_URL):
        self.url = url
        self.model = "local-model"  # LM Studio usually auto-detects

    def call(self, system: str, user: str, temperature: float = 0.2) -> str:
        """Call LLM and return response"""
        try:
            resp = requests.post(self.url, json={
                "model": self.model,
                "messages": [
                    {"role": "system", "content": system},
                    {"role": "user", "content": user}
                ],
                "temperature": temperature,
                "max_tokens": 4000
            }, timeout=120)

            if resp.status_code != 200:
                return f"ERROR: LLM API returned {resp.status_code}: {resp.text}"

            result = resp.json()
            return result["choices"][0]["message"]["content"]

        except requests.exceptions.ConnectionError:
            return "ERROR: Cannot connect to LM Studio. Is it running at http://127.0.0.1:1234?"
        except Exception as e:
            return f"ERROR: {str(e)}"

    def generate_patch(self, task: Task, context: str = "", previous_error: str = "") -> str:
        """Ask LLM to generate a code patch"""
        system = """You are the GVPIE build agent. Your job is to implement tasks for the GVPIE project.

Output ONLY a SEARCH/REPLACE block in this format:

<<<<<<< SEARCH
[exact code to find]
=======
[new code to replace with]
>>>>>>> REPLACE

Rules:
- Output ONE search/replace block per file
- Match existing indentation exactly
- Include enough context to make the search unique
- Do not output explanations, only the SEARCH/REPLACE block"""

        # Load file contents
        file_contents = []
        for fpath in task.inputs:
            full_path = REPO_ROOT / fpath
            if full_path.exists():
                content = full_path.read_text()
                file_contents.append(f"--- {fpath} ---\n{content}")

        user = f"""Task: {task.title}

Current code:
{chr(10).join(file_contents)}

{context}

{f'Previous attempt failed with error:{chr(10)}{previous_error}' if previous_error else ''}

Generate the SEARCH/REPLACE block to implement this task."""

        return self.call(system, user)

    def generate_pixel_program(self, task: Task) -> str:
        """Ask LLM to generate a pixel program spec"""
        system = """You are the GVPIE pixel architect. Output ONLY valid JSON with this structure:

{
  "kind": "pixel_program",
  "version": 1,
  "name": "descriptive_name",
  "canvas": {"width": 64, "height": 64},
  "palette": {
    "bg": [0, 0, 0],
    "fg": [255, 255, 255]
  },
  "ops": [
    {"op": "FILL", "color": "bg"},
    {"op": "SET", "x": 5, "y": 5, "color": "fg"},
    {"op": "RECT", "x": 0, "y": 0, "width": 10, "height": 10, "color": "fg"}
  ],
  "metadata": {"description": "what this does"}
}

Available ops: FILL, SET, RECT
Do not output code or explanations, only the JSON."""

        user = f"""Task: {task.title}

Generate a pixel program that implements this visual element."""

        return self.call(system, user, temperature=0.3)


class PatchApplier:
    """Applies LLM-generated patches to code"""

    @staticmethod
    def apply_search_replace(file_path: pathlib.Path, patch: str) -> bool:
        """Apply a SEARCH/REPLACE block to a file"""
        # Parse the patch
        if "<<<<<<< SEARCH" not in patch or ">>>>>>>" not in patch:
            print(f"⚠️  Invalid patch format")
            return False

        try:
            parts = patch.split("<<<<<<< SEARCH")[1].split("=======")
            search_block = parts[0].strip()
            replace_block = parts[1].split(">>>>>>> REPLACE")[0].strip()

            # Read file
            if not file_path.exists():
                print(f"⚠️  File not found: {file_path}")
                return False

            content = file_path.read_text()

            # Apply replacement
            if search_block not in content:
                print(f"⚠️  Search block not found in file")
                print(f"Looking for:\n{search_block[:200]}...")
                return False

            new_content = content.replace(search_block, replace_block)
            file_path.write_text(new_content)

            print(f"✅ Applied patch to {file_path}")
            return True

        except Exception as e:
            print(f"⚠️  Patch application error: {e}")
            return False


class TaskExecutor:
    """Executes build/test commands"""

    @staticmethod
    def run_command(cmd: List[str], cwd: pathlib.Path = REPO_ROOT) -> BuildRun:
        """Run a command and capture results"""
        run_id = f"run-{uuid.uuid4().hex[:8]}"

        try:
            result = subprocess.run(
                cmd,
                cwd=cwd,
                capture_output=True,
                text=True,
                timeout=300
            )

            return BuildRun(
                id=run_id,
                task_id="",  # Set by caller
                artifact_id="",  # Set by caller
                command=" ".join(cmd),
                exit_code=result.returncode,
                stdout=result.stdout,
                stderr=result.stderr
            )

        except subprocess.TimeoutExpired:
            return BuildRun(
                id=run_id,
                task_id="",
                artifact_id="",
                command=" ".join(cmd),
                exit_code=-1,
                stdout="",
                stderr="Command timed out after 300s"
            )
        except Exception as e:
            return BuildRun(
                id=run_id,
                task_id="",
                artifact_id="",
                command=" ".join(cmd),
                exit_code=-1,
                stdout="",
                stderr=str(e)
            )


class MemoryStore:
    """Stores tasks, artifacts, and results (LanceDB or JSON fallback)"""

    def __init__(self):
        if LANCEDB_AVAILABLE:
            self.db = lancedb.connect(DB_PATH)
            self._init_tables()
        else:
            self.db = None
            self.storage_dir = REPO_ROOT / "auto_gvpie_memory"
            self.storage_dir.mkdir(exist_ok=True)

    def _init_tables(self):
        """Initialize LanceDB tables"""
        if not self.db:
            return

        # Create tables if they don't exist
        if "tasks" not in self.db.table_names():
            self.db.create_table("tasks", data=[])
        if "artifacts" not in self.db.table_names():
            self.db.create_table("artifacts", data=[])
        if "runs" not in self.db.table_names():
            self.db.create_table("runs", data=[])

    def store_task(self, task: Task):
        """Store a task"""
        if self.db:
            tbl = self.db.open_table("tasks")
            tbl.add([asdict(task)])
        else:
            path = self.storage_dir / f"task_{task.id}.json"
            path.write_text(json.dumps(asdict(task), indent=2))

    def store_artifact(self, artifact: LLMArtifact):
        """Store an LLM artifact"""
        if self.db:
            tbl = self.db.open_table("artifacts")
            tbl.add([asdict(artifact)])
        else:
            path = self.storage_dir / f"artifact_{artifact.id}.json"
            path.write_text(json.dumps(asdict(artifact), indent=2))

    def store_run(self, run: BuildRun):
        """Store a build run"""
        if self.db:
            tbl = self.db.open_table("runs")
            tbl.add([asdict(run)])
        else:
            path = self.storage_dir / f"run_{run.id}.json"
            path.write_text(json.dumps(asdict(run), indent=2))


class AutoGVPIE:
    """Main autonomous development loop orchestrator"""

    def __init__(self):
        self.llm = LLMInterface()
        self.patcher = PatchApplier()
        self.executor = TaskExecutor()
        self.memory = MemoryStore()

    def load_tasks(self) -> List[Task]:
        """Load tasks from file"""
        if not TASKS_FILE.exists():
            # Create sample tasks
            sample = [
                Task(
                    id=f"task-{uuid.uuid4().hex[:8]}",
                    title="Add debug logging to GPU analyzer",
                    inputs=["ai_runtime_rust/src/gpu_ppl_analyzer.rs"],
                    tests=["cargo check --package ai-runtime"],
                    status="pending"
                )
            ]
            TASKS_FILE.write_text(json.dumps([asdict(t) for t in sample], indent=2))
            return sample

        data = json.loads(TASKS_FILE.read_text())
        return [Task(**t) for t in data]

    def save_tasks(self, tasks: List[Task]):
        """Save tasks to file"""
        TASKS_FILE.write_text(json.dumps([asdict(t) for t in tasks], indent=2))

    def process_one_task(self, task: Task) -> bool:
        """Process a single task through the full cycle"""
        print(f"\n{'='*60}")
        print(f"🎯 Task: {task.title}")
        print(f"{'='*60}\n")

        # Update status
        task.status = "in_progress"

        # Generate artifact
        print("🤖 Asking local LLM for implementation...")
        if "pixel" in task.title.lower() or "visual" in task.title.lower():
            response = self.llm.generate_pixel_program(task)
        else:
            response = self.llm.generate_patch(task)

        if response.startswith("ERROR:"):
            print(f"❌ LLM Error: {response}")
            task.status = "failed"
            return False

        # Store artifact
        artifact = LLMArtifact(
            id=f"art-{uuid.uuid4().hex[:8]}",
            task_id=task.id,
            kind="pixel_program" if "pixel" in task.title.lower() else "patch",
            content={"raw": response},
            embedding_text=f"{task.title} {response[:500]}",
            tags=["gvpie", "auto_generated"]
        )
        self.memory.store_artifact(artifact)

        print(f"💾 Stored artifact: {artifact.id}")

        # Apply patch (if it's code)
        if artifact.kind == "patch":
            print("🔧 Applying patch...")
            for fpath in task.inputs:
                if not self.patcher.apply_search_replace(REPO_ROOT / fpath, response):
                    task.status = "failed"
                    return False
        else:
            # Save pixel program
            pixel_path = ARTIFACTS_DIR / f"{artifact.id}.json"
            pixel_path.write_text(json.dumps(json.loads(response), indent=2))
            print(f"💾 Saved pixel program: {pixel_path}")

        # Run tests
        print("🧪 Running tests...")
        success = True
        for test_cmd in task.tests:
            cmd = test_cmd.split()
            run = self.executor.run_command(cmd)
            run.task_id = task.id
            run.artifact_id = artifact.id

            self.memory.store_run(run)

            if run.exit_code != 0:
                print(f"❌ Test failed: {test_cmd}")
                print(f"   stderr: {run.stderr[:500]}")
                success = False

                # Try to fix
                print("🔄 Asking LLM to fix error...")
                fix_response = self.llm.generate_patch(task, previous_error=run.stderr)

                # TODO: Apply fix and retry
                break
            else:
                print(f"✅ Test passed: {test_cmd}")

        if success:
            task.status = "done"
            print(f"\n🎉 Task completed: {task.title}")
        else:
            task.status = "failed"
            print(f"\n⚠️  Task failed: {task.title}")

        return success

    def run_loop(self, max_tasks: int = 10):
        """Run the autonomous development loop"""
        print("🚀 Auto-GVPIE: Starting autonomous development loop")
        print(f"   LM Studio: {LM_STUDIO_URL}")
        print(f"   Repository: {REPO_ROOT}")
        print(f"   LanceDB: {'✅ Available' if LANCEDB_AVAILABLE else '❌ Using file storage'}\n")

        tasks = self.load_tasks()
        pending = [t for t in tasks if t.status == "pending"]

        print(f"📋 Found {len(pending)} pending tasks\n")

        completed = 0
        for task in pending[:max_tasks]:
            if self.process_one_task(task):
                completed += 1

            # Save progress
            self.save_tasks(tasks)

        print(f"\n{'='*60}")
        print(f"✅ Completed {completed}/{len(pending)} tasks")
        print(f"{'='*60}\n")


def main():
    """Entry point"""
    import argparse

    parser = argparse.ArgumentParser(description="Auto-GVPIE autonomous development loop")
    parser.add_argument("--once", action="store_true", help="Run once then exit")
    parser.add_argument("--max-tasks", type=int, default=10, help="Max tasks per run")
    args = parser.parse_args()

    auto = AutoGVPIE()
    auto.run_loop(max_tasks=args.max_tasks)


if __name__ == "__main__":
    main()
