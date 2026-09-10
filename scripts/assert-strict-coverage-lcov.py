#!/usr/bin/env python3
import sys
from pathlib import Path


def strict_coverage_failures(report: str) -> list[str]:
    failures: list[str] = []
    source_file: str | None = None
    line_counts: list[int] = []
    function_found: int | None = None
    function_hit: int | None = None
    record_count = 0

    def finish_record() -> None:
        nonlocal source_file, line_counts, function_found, function_hit, record_count
        if source_file is None:
            return
        if not line_counts:
            failures.append(f"coverage file has no line data: {source_file}")
        elif any(count == 0 for count in line_counts):
            failures.append(f"coverage lines must be 100%: {source_file}")
        if function_found is None or function_hit is None:
            failures.append(f"coverage file has no function totals: {source_file}")
        elif function_found != function_hit:
            failures.append(
                f"coverage functions must be 100%: {source_file}:{function_hit}/{function_found}"
            )
        source_file = None
        line_counts = []
        function_found = None
        function_hit = None
        record_count += 1

    for raw_line in report.splitlines():
        if raw_line.startswith("SF:"):
            finish_record()
            source_file = raw_line[3:]
        elif raw_line.startswith("DA:"):
            try:
                _, count = raw_line[3:].rsplit(",", 1)
                line_counts.append(int(count))
            except ValueError:
                failures.append(f"coverage line data is malformed: {source_file}")
        elif raw_line.startswith("FNF:"):
            try:
                function_found = int(raw_line[4:])
            except ValueError:
                failures.append(f"coverage function totals are malformed: {source_file}")
        elif raw_line.startswith("FNH:"):
            try:
                function_hit = int(raw_line[4:])
            except ValueError:
                failures.append(f"coverage function totals are malformed: {source_file}")
        elif raw_line == "end_of_record":
            finish_record()
    finish_record()
    if record_count == 0:
        failures.append("coverage report has no source files")
    return failures


def self_test() -> int:
    good = "SF:src/lib.rs\nDA:1,2\nFNF:1\nFNH:1\nend_of_record\n"
    bad_line = "SF:src/lib.rs\nDA:1,0\nFNF:1\nFNH:1\nend_of_record\n"
    bad_function = "SF:src/lib.rs\nDA:1,2\nFNF:2\nFNH:1\nend_of_record\n"
    if strict_coverage_failures(good):
        print("strict LCOV self-test rejected covered data", file=sys.stderr)
        return 1
    if not strict_coverage_failures(bad_line):
        print("strict LCOV self-test accepted uncovered line data", file=sys.stderr)
        return 1
    if not strict_coverage_failures(bad_function):
        print("strict LCOV self-test accepted uncovered function data", file=sys.stderr)
        return 1
    if not strict_coverage_failures(""):
        print("strict LCOV self-test accepted an empty report", file=sys.stderr)
        return 1
    return 0


def main() -> int:
    if sys.argv[1:] == ["--self-test"]:
        return self_test()
    if len(sys.argv) != 2:
        print("usage: assert-strict-coverage-lcov.py <coverage.lcov>", file=sys.stderr)
        return 2
    try:
        failures = strict_coverage_failures(Path(sys.argv[1]).read_text(encoding="utf-8"))
    except OSError as error:
        print(f"strict LCOV report could not be read: {error}", file=sys.stderr)
        return 1
    if failures:
        print("\n".join(failures), file=sys.stderr)
        return 1
    print("strict LCOV coverage passed")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
