import sqlite3
import time
import hashlib
from pathlib import Path
from datetime import datetime
from pxl_os.cartridge_manager import CartridgeManager, CartridgeType, CartridgeMetadata
from pxl_os.gvpie_bridge import GVPIEBridge

class ZHIDaemon:
    def __init__(self):
        self.db_path = "daemon.db"
        self.conn = sqlite3.connect(self.db_path)
        self.setup_database()

        self.gvpie_bridge = GVPIEBridge()
        self.cartridge_manager = CartridgeManager(self.gvpie_bridge)

    def setup_database(self):
        """Sets up the database schema."""
        cursor = self.conn.cursor()
        cursor.execute("""
            CREATE TABLE IF NOT EXISTS improvement_scripts (
                id INTEGER PRIMARY KEY,
                name TEXT,
                lang TEXT,
                purpose TEXT,
                code TEXT,
                requires_caps TEXT,
                validator TEXT,
                enabled INTEGER DEFAULT 1
            )
        """)

        # Add columns if they don't exist
        for column, dtype in [
            ("cartridge_path", "TEXT"),
            ("cartridge_type", "TEXT"),
            ("cartridge_metadata", "TEXT"),
        ]:
            try:
                cursor.execute(f"ALTER TABLE improvement_scripts ADD COLUMN {column} {dtype}")
            except sqlite3.OperationalError as e:
                if "duplicate column name" not in str(e):
                    raise

        cursor.execute("""
            CREATE TABLE IF NOT EXISTS cartridge_registry (
                id INTEGER PRIMARY KEY,
                path TEXT UNIQUE,
                type TEXT,
                version TEXT,
                width INTEGER,
                height INTEGER,
                pages INTEGER,
                checksum TEXT,
                created_at TEXT,
                tags TEXT,
                executed_count INTEGER DEFAULT 0,
                last_executed TEXT
            )
        """)
        self.conn.commit()

    def run_pending_improvements(self, max_runs=2):
        """ENHANCED: Support cartridge execution"""
        cur = self.conn.cursor()
        cur.execute("""
            SELECT id, name, lang, purpose, code, requires_caps, validator, cartridge_path
            FROM improvement_scripts
            WHERE enabled = 1
            ORDER BY id LIMIT ?
        """, (max_runs,))

        for row in cur.fetchall():
            sid, name, lang, purpose, code, requires_caps, validator, cartridge_path = row

            if lang == "python":
                self.execute_one(row)
            elif lang == "sql":
                self.execute_one(row)
            elif lang == "gvpie":
                self._execute_gvpie_program(sid, name, code)
            elif lang == "cartridge":
                self._execute_cartridge(sid, name, cartridge_path)
            elif lang == "gvpie_gen":
                self._generate_cartridge(sid, name, code)

        self.conn.commit()

    def execute_one(self, row):
        # Placeholder for existing python/sql execution logic
        pass

    def _execute_gvpie_program(self, sid: int, name: str, code: str):
        """Execute text-based GVPIE program"""
        try:
            for i, line in enumerate(code.split('\n')):
                self.gvpie_bridge.send_command(f"TXT 0 {i} {line}")

            # This is a placeholder as the output is not yet implemented
            canvas = np.zeros((64, 128, 3), dtype=np.uint8)

            metadata = CartridgeMetadata(
                type=CartridgeType.PIXEL,
                version="1.0",
                width=128,
                height=64,
                pages=1,
                encoding="rgb",
                checksum=self._compute_checksum(canvas),
                created_at=datetime.now().isoformat(),
                tags=[name, "gvpie", "generated"]
            )

            path = Path(f"cartridges/gvpie_{name}.png")
            self.cartridge_manager.save_cartridge(canvas, metadata, path)

        except Exception as e:
            print(f"Error executing GVPIE program: {e}")

    def _execute_cartridge(self, sid: int, name: str, cartridge_path: str):
        """Load and execute existing cartridge"""
        try:
            path = Path(cartridge_path)
            result_canvas = self.cartridge_manager.execute_cartridge(path)

            output_path = Path(f"cartridges/{name}_output.png")
            metadata = CartridgeMetadata(
                type=CartridgeType.PIXEL,
                version="1.0",
                width=128,
                height=64,
                pages=1,
                encoding="rgb",
                checksum=self._compute_checksum(result_canvas),
                created_at=datetime.now().isoformat(),
                tags=[name, "output"]
            )
            self.cartridge_manager.save_cartridge(result_canvas, metadata, output_path)

        except Exception as e:
            print(f"Error executing cartridge: {e}")

    def _generate_cartridge(self, sid: int, name: str, description: str):
        """Generate cartridge from natural language description"""
        program = f"TXT 5 5 {description}"
        self._execute_gvpie_program(sid, name, program)

    def _compute_checksum(self, canvas: np.ndarray) -> str:
        """Compute SHA256 checksum of canvas"""
        return hashlib.sha256(canvas.tobytes()).hexdigest()[:16]

from flask import Flask, jsonify, request

class ZHIDaemon:
    def __init__(self):
        self.db_path = "daemon.db"
        self.conn = sqlite3.connect(self.db_path, check_same_thread=False)
        self.setup_database()

        self.gvpie_bridge = GVPIEBridge()
        self.cartridge_manager = CartridgeManager(self.gvpie_bridge)

        self.app = Flask(__name__)
        self.setup_routes()

    def setup_routes(self):
        @self.app.route("/cartridges", methods=["GET"])
        def list_cartridges_api():
            type_filter = request.args.get("type")
            cartridges = self.cartridge_manager.list_cartridges(type_filter)
            return jsonify([str(p) for p in cartridges])

        @self.app.route("/cartridges/execute/<path:cartridge_path>", methods=["POST"])
        def execute_cartridge_api(cartridge_path):
            try:
                result = self.cartridge_manager.execute_cartridge(Path(cartridge_path))
                return jsonify({"success": True, "result": result.tolist()})
            except Exception as e:
                return jsonify({"success": False, "error": str(e)}), 500

        @self.app.route("/cartridges/search", methods=["GET"])
        def search_cartridges_api():
            query = request.args.get("q")
            results = self.cartridge_manager.search_cartridges(query)
            return jsonify([str(p) for p in results])

    def run_api(self, host="0.0.0.0", port=5000):
        self.app.run(host=host, port=port)
