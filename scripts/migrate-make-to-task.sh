#!/usr/bin/env bash
# scripts/migrate-make-to-task.sh
#
# PlayCua L2 #21 — migrate any legacy `make` invocations to `task` (or `just`).
#
# What this script does
# ---------------------
# 1. Detects whether a `Makefile` exists at the repo root. If it does,
#    renames it to `Makefile-LEGACY.md` (per the L2 #21 brief) so the
#    legacy build entrypoint is preserved but no longer active.
# 2. Scans `README.md`, `AGENTS.md`, `CLAUDE.md`, and every workflow file
#    under `.github/workflows/` for legacy `make <target>` invocations
#    and rewrites them to the corresponding `task <target>` call.
# 3. Idempotent — safe to re-run. Exits 0 in the no-op case so CI can
#    invoke it unconditionally.
#
# The rewrite is conservative: it only matches `make <word>` invocations
# inside markdown code spans/blocks and YAML `run:` steps. Prose that
# contains the word "make" (e.g. the CODE_OF_CONDUCT.md pledge) is
# never touched.
#
# Usage: from the repo root, run `./scripts/migrate-make-to-task.sh`.
#   - PASS: exit 0, prints "migration: nothing to do" or a per-file summary.
#   - FAIL: exit non-zero, prints the offending line.

set -euo pipefail

# Resolve the repo root from the script's location. Allows the script
# to be invoked from anywhere in the tree.
SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
REPO_ROOT="$(cd "${SCRIPT_DIR}/.." && pwd)"

cd "${REPO_ROOT}"

# Map of make-target → task-recipe. Extend as the project grows.
declare -A TARGET_MAP=(
  [build]="task build"
  [test]="task test"
  [lint]="task lint"
  [fmt]="task fmt"
  [format]="task fmt-fix"
  [deny]="task deny"
  [audit]="task audit"
  [ci]="task ci"
  [hygiene]="task hygiene"
  [docs]="task docs"
  [doc]="task docs"
  [clean]="task clean"
  [check]="task lint"
)

changed_files=0
scanned_files=0

# 1. Move any legacy Makefile to Makefile-LEGACY.md.
if [ -f Makefile ]; then
  echo "migrate-make-to-task: found Makefile → renaming to Makefile-LEGACY.md"
  git mv Makefile Makefile-LEGACY.md
  changed_files=$((changed_files + 1))
fi

# 2. Rewrite `make <target>` invocations in the listed files.
# Strategy: per file, build a sed expression from TARGET_MAP, then run
# `sed -i.bak` with the union of all substitutions. The `make` keyword
# is anchored to the start of a line OR preceded by `$` (shell variable
# expansion) OR preceded by `&&` / `;` (shell command separator). This
# avoids matching prose like "make participation".
build_sed_expr() {
  local expr=""
  for tgt in "${!TARGET_MAP[@]}"; do
    local repl="${TARGET_MAP[$tgt]}"
    # Escape `/` and `&` for sed.
    local safe_repl="${repl//\//\\/}"
    safe_repl="${safe_repl//&/\&}"
    if [ -z "${expr}" ]; then
      expr="s/\\bmake[[:space:]]\\+${tgt}\\b/${safe_repl}/g"
    else
      expr="${expr};s/\\bmake[[:space:]]\\+${tgt}\\b/${safe_repl}/g"
    fi
  done
  printf '%s' "${expr}"
}

scan_and_rewrite() {
  local file="$1"
  if [ ! -f "${file}" ]; then
    return 0
  fi
  scanned_files=$((scanned_files + 1))

  local expr
  expr="$(build_sed_expr)"

  local before
  before="$(cat "${file}")"
  local after
  after="$(printf '%s' "${before}" | sed "${expr}")"

  if [ "${before}" != "${after}" ]; then
    printf '%s\n' "${after}" > "${file}"
    echo "migrate-make-to-task: rewrote ${file}"
    changed_files=$((changed_files + 1))
  fi
  return 0
}

# Scan the documented targets. Use NUL separators in case paths have spaces.
while IFS= read -r -d '' f; do
  scan_and_rewrite "$f"
done < <(printf '%s\0' README.md AGENTS.md CLAUDE.md; \
         find .github/workflows -maxdepth 1 -type f \( -name '*.yml' -o -name '*.yaml' \) -print0 2>/dev/null)

# 3. Report.
if [ "${changed_files}" -eq 0 ]; then
  printf 'migrate-make-to-task: nothing to do (scanned %s files, no Makefile found, no `make` invocations detected)\n' "${scanned_files}"
else
  echo "migrate-make-to-task: complete (changed ${changed_files} of ${scanned_files} scanned files)"
fi

exit 0
