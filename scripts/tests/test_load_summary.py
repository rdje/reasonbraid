"""The load harness's summary (SIGNOFF-REPAIR.11.3.4): exact counts, percentiles over
the committed requests only, and a PASS that means every requested command committed."""

from pathlib import Path
import sys
import unittest

sys.path.insert(0, str(Path(__file__).resolve().parents[1]))
import load_summary as summary


class SummaryTests(unittest.TestCase):
    def test_failures_do_not_pull_the_percentiles(self):
        # Ten commits at 0.100s and ten fast refusals at 0.001s: over EVERY line
        # (the old formula) p50 would be 0.001s; over the committed ones it is 0.100s.
        lines = ["200 0.100"] * 10 + ["409 0.001"] * 10
        passed, report = summary.summarize(lines, 20, 0, 1.0)
        self.assertFalse(passed)
        text = "\n".join(report)
        self.assertIn("p50 0.1000s", text)
        self.assertIn("p95 0.1000s", text)
        self.assertIn("failures: 10", text)
        self.assertIn("FAIL: 10 commands did not commit with 200", text)

    def test_a_run_must_be_exactly_the_requested_count(self):
        passed, report = summary.summarize(["200 0.01"] * 16, 10, 0, 1.0)
        self.assertFalse(passed)
        self.assertIn("FAIL: 16 of 10 commands ran", "\n".join(report))
        passed, report = summary.summarize(["200 0.01"] * 9, 10, 0, 1.0)
        self.assertFalse(passed)

    def test_nothing_run_is_not_a_pass(self):
        passed, report = summary.summarize([], 1, 0, 1.0)
        self.assertFalse(passed)
        self.assertIn("no committed request, so no percentile", "\n".join(report))

    def test_an_unanswered_request_is_a_failure_and_a_failed_worker_fails_the_run(self):
        passed, _ = summary.summarize(["200 0.01", "000 30.001"], 2, 0, 31.0)
        self.assertFalse(passed)
        passed, report = summary.summarize(["200 0.01"] * 4, 4, 1, 1.0)
        self.assertFalse(passed)
        self.assertIn("FAIL: 1 workers did not exit cleanly", "\n".join(report))

    def test_a_clean_run_passes_with_nearest_rank_percentiles(self):
        lines = [f"200 {i / 100:.2f}" for i in range(1, 21)]  # 0.01 … 0.20
        passed, report = summary.summarize(lines, 20, 0, 2.0)
        self.assertTrue(passed, report)
        text = "\n".join(report)
        self.assertIn("p50 0.1000s", text)   # rank ceil(0.5·20) = 10 → 0.10
        self.assertIn("p95 0.1900s", text)   # rank ceil(0.95·20) = 19 → 0.19
        self.assertIn("10.0 committed commands/s", text)

    def test_a_run_of_nothing_requested_is_unusable_input(self):
        for requested in (0, -3):
            with self.assertRaises(ValueError):
                summary.summarize([], requested, 0, 1.0)

    def test_a_malformed_line_is_unusable_input(self):
        with self.assertRaises(ValueError):
            summary.summarize(["200"], 1, 0, 1.0)


if __name__ == "__main__":
    unittest.main()
