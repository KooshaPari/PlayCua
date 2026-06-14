#!/usr/bin/env bash
# scripts/coverage.sh — PlayCua L3 #41 coverage runner
#
# Drives `cargo llvm-cov` to produce an HTML coverage report and asserts
# the workspace's line coverage is at or above the L3 #41 floor of 80%.
#
# Inputs (env):
#   PLAYCUA_COV_MIN   minimum line coverage % (default: 80.0)
#   PLAYCUA_COV_KIND  "html" (default) or "lcov" — what report to render
#   CARGO             cargo binary (default: cargo on $PATH)
#
# Exit codes:
#   0  coverage met or exceeded the floor
#   1  coverage below the floor (threshold miss)
#   2  toolchain / invocation error (e.g. cargo-llvm-cov not installed)
#   3  could not parse the summary line (defensive — the regex assumes
#      cargo-llvm-cov's stable terminal output format)
#
# What this script does NOT do:
#   * It does NOT install `cargo-llvm-cov`. Install it with
#     `cargo install cargo-llvm-cov --locked` before running.
#   * It does NOT run `cargo build` first; cargo-llvm-cov builds
#     the instrumented test binaries itself under the `coverage`
#     profile, which the `[profile.coverage]` table in Cargo.toml
#     and the `rustflags` in `.cargo/config.toml` configure.
#   * It does NOT push coverage reports anywhere; the HTML report
#     lands in `coverage/html/index.html` and is gitignored.
#
# Why a shell script (and not a `just` recipe):
#   The `just` / `task` recipes in `justfile` and `Taskfile.yml`
#   intentionally stay at the level of `cargo test`, `cargo clippy`,
#   etc. — building-block invocations. Coverage is a *composed*
#   workflow (build + run tests + post-process counters + parse
#   the summary line), so it lives here so both `just` recipes and
#   CI workflows can call it without duplicating the parse logic.

set -euo pipefail

# ---------------------------------------------------------------------------
# Resolve paths
# ---------------------------------------------------------------------------
SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
REPO_ROOT="$(cd "${SCRIPT_DIR}/.." && pwd)"

# All cargo-llvm-cov invocations must run from the workspace root
# (the script lives at `<repo>/scripts/...`, so REPO_ROOT is one
# level up). cd is the most direct way to be sure `--workspace` and
# `--manifest-path` both agree.
cd "${REPO_ROOT}"

# ---------------------------------------------------------------------------
# Config
# ---------------------------------------------------------------------------
MIN_COV="${PLAYCUA_COV_MIN:-80.0}"
KIND="${PLAYCUA_COV_KIND:-html}"
CARGO_BIN="${CARGO:-cargo}"
COVERAGE_DIR="${REPO_ROOT}/coverage"

# `--no-clean` is the default: cargo-llvm-cov removes stale profraw
# files between runs (it's documented to be safe). We let the tool
# handle it.
#
# `--workspace` is required: this script is the workspace coverage
# entry point. A future per-crate variant can add `--package <name>`.
#
# `--output-dir coverage/` matches the `[workspace.metadata.coverage]
# output-dir = "coverage"` setting in Cargo.toml.
LLVMCOV_ARGS=(
    --workspace
    "--output-dir" "${COVERAGE_DIR}/"
)

case "${KIND}" in
    html)
        LLVMCOV_ARGS+=(--html)
        ;;
    lcov)
        LLVMCOV_ARGS+=(--lcov --output-path "${COVERAGE_DIR}/lcov.info")
        ;;
    *)
        echo "coverage.sh: unknown PLAYCUA_COV_KIND='${KIND}' (expected: html|lcov)" >&2
        exit 2
        ;;
esac

# ---------------------------------------------------------------------------
# Preflight
# ---------------------------------------------------------------------------
# We require cargo-llvm-cov on $PATH. The `--version` invocation is
# cheap (it just prints the version and exits 0) and gives us a
# useful error message if the tool is missing.
if ! "${CARGO_BIN}" llvm-cov --version >/dev/null 2>&1; then
    cat >&2 <<EOF
coverage.sh: cargo-llvm-cov is not installed on the cargo subcommand path.

Install it with:

    ${CARGO_BIN} install cargo-llvm-cov --locked

(See https://github.com/taiki-e/cargo-llvm-cov#installation for the
upstream install instructions and minimum Rust toolchain version.)
EOF
    exit 2
fi

# ---------------------------------------------------------------------------
# Run cargo llvm-cov and capture the textual summary
# ---------------------------------------------------------------------------
# `--summary-only` would suppress the HTML report we just asked for,
# so we instead capture the full output and grep the summary line
# out of it. `tee` lets us both display the report to the user
# (so CI logs show the full coverage table) and parse it.
SUMMARY_LOG="$(mktemp -t playcua-cov.XXXXXX)"
trap 'rm -f "${SUMMARY_LOG}"' EXIT

# Note: we deliberately do NOT use `set -x` here — `cargo llvm-cov`
# is slow, and tracing the full command lines floods CI logs.
echo "coverage.sh: running: ${CARGO_BIN} llvm-cov ${LLVMCOV_ARGS[*]}"
echo "coverage.sh: minimum line coverage: ${MIN_COV}%"
echo

# `cargo llvm-cov` writes the textual summary (the same line the
# terminal shows) to stdout AFTER the HTML report is generated.
# The HTML output goes to `${COVERAGE_DIR}/html/`, so capturing
# stdout is safe.
"${CARGO_BIN}" llvm-cov "${LLVMCOV_ARGS[@]}" 2>&1 | tee "${SUMMARY_LOG}"

# ---------------------------------------------------------------------------
# Parse the summary line
# ---------------------------------------------------------------------------
# cargo-llvm-cov prints a line like:
#
#     ... 81.42% line coverage ...
#
# or, when the report includes branch coverage:
#
#     ... 81.42% line coverage (+0.50%) 90.12% region coverage ...
#
# The regex is intentionally permissive: we look for "<float>% line
# coverage" anywhere in the line. The first match wins.
SUMMARY_LINE="$(grep -E '[0-9]+(\.[0-9]+)?% line coverage' "${SUMMARY_LOG}" | head -n1 || true)"

if [[ -z "${SUMMARY_LINE}" ]]; then
    echo >&2
    echo "coverage.sh: could not find a '...% line coverage' line in the cargo-llvm-cov output." >&2
    echo "coverage.sh: this usually means cargo-llvm-cov is too old to emit a summary line" >&2
    echo "coverage.sh: (try \`cargo install cargo-llvm-cov --locked --force\`) or the tool" >&2
    echo "coverage.sh: encountered an error mid-run (re-run with \`--no-clean --show-output\`" >&2
    echo "coverage.sh: to see the full report)." >&2
    echo "coverage.sh: captured output was:" >&2
    sed 's/^/    /' "${SUMMARY_LOG}" >&2 || true
    exit 3
fi

# Extract the first percentage on the matched line.
PARSED_PCT="$(echo "${SUMMARY_LINE}" | grep -oE '[0-9]+(\.[0-9]+)?%' | head -n1 | tr -d '%')"

if [[ -z "${PARSED_PCT}" ]]; then
    echo "coverage.sh: matched line '${SUMMARY_LINE}' but could not extract a number." >&2
    exit 3
fi

# ---------------------------------------------------------------------------
# Assert
# ---------------------------------------------------------------------------
# Use `awk` to do the floating-point comparison (bash can't do FP
# natively). We compare `parsed >= min` and exit 0 / 1 accordingly.
if awk -v a="${PARSED_PCT}" -v b="${MIN_COV}" 'BEGIN { exit !(a+0 >= b+0) }'; then
    echo
    echo "coverage.sh: PASS — ${PARSED_PCT}% line coverage >= ${MIN_COV}% (floor for L3 #41)"
    echo "coverage.sh: HTML report: ${COVERAGE_DIR}/html/index.html"
    exit 0
else
    echo
    echo "coverage.sh: FAIL — ${PARSED_PCT}% line coverage < ${MIN_COV}% (floor for L3 #41)" >&2
    echo "coverage.sh: HTML report (for triage): ${COVERAGE_DIR}/html/index.html" >&2
    echo "coverage.sh: inspect uncovered lines in the report, add tests, then re-run." >&2
    exit 1
fi
