"""The demonstration's check shapes (SIGNOFF-REPAIR.11.3.7): a negative check and an
evidence capture must not pass on a failed command."""

import http.server
from pathlib import Path
import socketserver
import subprocess
import sys
import tempfile
import threading
import unittest

sys.path.insert(0, str(Path(__file__).resolve().parents[1]))
import project_env

ROOT = Path(__file__).resolve().parents[2]
LIB = ROOT / "scripts" / "lib" / "demo_checks.sh"


def bash(script: str) -> int:
    return subprocess.run(["bash", "-c", f". {LIB}\n{script}"], capture_output=True, timeout=60).returncode


class AbsentTests(unittest.TestCase):
    def test_the_old_negated_pipeline_passed_on_a_failed_command(self):
        # The RED this leaf exists for, pinned: the command fails, and the check holds.
        self.assertEqual(bash("fails() { return 1; }\n! fails | grep -q revision_submitted"), 0)

    def test_absent_refuses_a_failed_command(self):
        self.assertNotEqual(bash("fails() { return 1; }\nabsent revision_submitted fails"), 0)

    def test_absent_refuses_a_failed_command_even_when_it_printed(self):
        # The realistic failure: the CLI prints an error body and exits non-zero. A
        # silent failure is also refused by the no-output rule, which is why this
        # case needs its own control (a mutant ignoring the status survived without it).
        self.assertNotEqual(
            bash("loud() { echo '{\"error\":\"the server is unavailable\"}'; return 1; }\n"
                 "absent revision_submitted loud"),
            0,
        )

    def test_absent_refuses_a_command_with_no_output(self):
        self.assertNotEqual(bash("silent() { return 0; }\nabsent revision_submitted silent"), 0)

    def test_absent_judges_the_output_of_a_command_that_succeeded(self):
        self.assertEqual(bash("say() { echo 'thread.created'; }\nabsent revision_submitted say"), 0)
        self.assertNotEqual(bash("say() { echo 'x revision_submitted y'; }\nabsent revision_submitted say"), 0)

    def test_a_pattern_that_looks_like_an_option_is_a_pattern(self):
        self.assertNotEqual(bash("say() { echo '-q is here'; }\nabsent -q say"), 0)


class Answer(http.server.BaseHTTPRequestHandler):
    def do_GET(self):
        status = int(self.path.strip("/") or "200")
        self.send_response(status)
        self.end_headers()
        self.wfile.write(f"body of {status}".encode())

    def log_message(self, *args):
        pass


class FetchTests(unittest.TestCase):
    @classmethod
    def setUpClass(cls):
        cls.server = socketserver.TCPServer(("127.0.0.1", 0), Answer)
        cls.port = cls.server.server_address[1]
        cls.thread = threading.Thread(target=cls.server.serve_forever, daemon=True)
        cls.thread.start()

    @classmethod
    def tearDownClass(cls):
        cls.server.shutdown()
        cls.server.server_close()

    def setUp(self):
        parent = project_env.local_directory(project_env.ROOT, "target/demo-check-tests")
        self.dir = Path(tempfile.mkdtemp(dir=parent))

    def tearDown(self):
        import shutil
        shutil.rmtree(self.dir)

    def fetch(self, status: int) -> tuple[int, str]:
        out = self.dir / f"{status}.txt"
        rc = bash(f"fetch '{out}' 'http://127.0.0.1:{self.port}/{status}'")
        return rc, out.read_text() if out.exists() else ""

    def test_a_2xx_capture_succeeds_and_keeps_the_body(self):
        self.assertEqual(self.fetch(200), (0, "body of 200"))

    def test_a_non_2xx_capture_fails_and_still_keeps_the_body(self):
        for status in (401, 404, 500):
            rc, body = self.fetch(status)
            self.assertNotEqual(rc, 0, status)
            self.assertEqual(body, f"body of {status}")

    def test_an_unreachable_server_fails(self):
        out = self.dir / "none.txt"
        self.assertNotEqual(bash(f"fetch '{out}' 'http://127.0.0.1:1/'"), 0)


class DemoShapeTests(unittest.TestCase):
    """Neither replaced shape may come back into the demonstration."""

    def test_no_negated_pipeline_and_no_unchecked_capture(self):
        import re
        demo = (ROOT / "scripts" / "demo_two_host.sh").read_text()
        code = "\n".join(l for l in demo.splitlines() if not l.lstrip().startswith("#"))
        negated = re.compile("[\"']!\\s")
        capture = re.compile(r'curl\s[^\n]*(?:\\\n[^\n]*)*>\s*"\$EVIDENCE/')
        self.assertIsNone(negated.search(code), "a negated pipeline in a probe string")
        self.assertIsNone(capture.search(code), "an unchecked capture into the evidence")
        self.assertIn('. "$ROOT/scripts/lib/demo_checks.sh"', code)
        poll = code[code.index("probe_poll() {"):code.index("export -f probe_poll")]
        self.assertIn("%{http_code}", poll, "the authenticated poll probe checks its status")
        self.assertIn('= "200" ]', poll)


if __name__ == "__main__":
    unittest.main()
