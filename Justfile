# ============================================================
# katana-ui-core - Development Justfile
# ============================================================
# Stable local, CI/CD, and release task entrypoint.
# Usage:
#   just
#   just <recipe>
#   just VERSION=vX.Y.Z release-check
# ============================================================

set shell := ["bash", "-uc"]

REPO_ROOT := justfile_directory()
RTK := env_var_or_default("RTK", `command -v rtk 2> /dev/null || true`)
RTK_CMD := if RTK == "" { "" } else { RTK + " " }
JOBS := env_var_or_default("JOBS", "2")
CARGO := env_var_or_default("CARGO", RTK_CMD + "cargo")
COVERAGE_BUILD_JOBS := env_var_or_default("CARGO_BUILD_JOBS", JOBS)
COVERAGE_TEST_THREADS := env_var_or_default("COVERAGE_TEST_THREADS", "8")
KUC_WORKSPACE_PACKAGES := "-p katana-ui-core -p katana-ui-core-storybook -p kuc-consumer-app"
KUC_FORMAT_PACKAGES := "-p katana-ui-core -p katana-ui-core-storybook -p kuc-consumer-app"
VERSION := env_var_or_default("VERSION", `awk -F '"' '/^version = / { print $2; exit }' Cargo.toml`)
VERSION_BARE := replace(VERSION, "v", "")
COVERAGE_MIN_LINES := "100"
COVERAGE_IMAGE := "katana-ui-core-coverage:rust-1.97.1"
RELEASE_REPO := env_var_or_default("RELEASE_REPO", "HiroyukiFuruno/katana-ui-core")
KAL_VERSION := env_var_or_default("KAL_VERSION", "0.5.1")

default: help

# Show this help
help:
    @just --list --unsorted

# Apply Rust formatting
fmt:
    {{CARGO}} fmt {{KUC_FORMAT_PACKAGES}}

# Check Rust formatting
fmt-check:
    {{CARGO}} fmt {{KUC_FORMAT_PACKAGES}} -- --check

# Check workspace type safety
check-types:
    {{CARGO}} check --workspace --locked

# Run strict Clippy checks
lint:
    {{CARGO}} clippy -j {{JOBS}} -p katana-ui-core --lib --all-features --locked -- -D warnings -D clippy::unwrap_used -D clippy::expect_used -D clippy::todo -D clippy::unimplemented -D clippy::dbg_macro -D clippy::panic -D clippy::wildcard_imports
    {{CARGO}} clippy -j {{JOBS}} -p katana-ui-core --tests --no-default-features --locked -- -D warnings -D clippy::unwrap_used -D clippy::expect_used -D clippy::todo -D clippy::unimplemented -D clippy::dbg_macro -D clippy::panic -D clippy::wildcard_imports
    {{CARGO}} clippy -j {{JOBS}} -p katana-ui-core --all-targets --all-features --locked -- -D warnings
    {{CARGO}} clippy -j {{JOBS}} -p katana-ui-core-storybook -p kuc-consumer-app --all-targets --all-features --locked -- -D warnings -D clippy::unwrap_used -D clippy::expect_used -D clippy::todo -D clippy::unimplemented -D clippy::dbg_macro -D clippy::panic -D clippy::wildcard_imports

# Install shared KatanA AST lint CLI from crates.io
ast-lint-install:
    {{CARGO}} install katana-ast-lint --version {{KAL_VERSION}} --locked --force

# Run shared KatanA Rust syntax checks
ast-lint:
    python3 scripts/assert-storybook-ui-harness.py
    kal check
    python3 scripts/assert-kuc-state-ownership.py
    python3 scripts/assert-kuc-guardrails.py
    python3 scripts/assert-storybook-page-layout.py

# Run the non-Storybook consumer contract used by release readiness.
consumer-app-contract:
    {{CARGO}} test -p kuc-consumer-app --locked
    {{CARGO}} test -p katana-ui-core --test generic_rust_app_contract --locked
    {{CARGO}} test -p katana-ui-core --test generic_rust_app_layout_contract --locked
    {{CARGO}} test -p katana-ui-core --test generic_rust_app_action_contract --locked

# Run katana-ui-core specific guardrails
kuc-guardrails: consumer-app-contract
    python3 scripts/test_kuc_guardrails.py
    python3 scripts/test_next_storybook_page_change.py
    python3 scripts/coverage/run-test-binaries.py --self-test
    python3 scripts/test_coverage_ci_storage.py
    python3 scripts/test_storybook_reflection_audit.py
    python3 scripts/test_storybook_ui_harness.py
    python3 scripts/test_storybook_ui_harness_public_options.py
    python3 scripts/test_storybook_ui_harness_public_options_feedback.py
    python3 scripts/test_storybook_ui_harness_public_options_specialized.py
    python3 scripts/test_storybook_manual_acceptance_queue.py
    python3 scripts/test_storybook_manual_acceptance_review.py
    python3 scripts/test_storybook_manual_acceptance_status.py
    python3 scripts/test_storybook_manual_acceptance_next.py
    python3 scripts/test_storybook_manual_acceptance_approval_template.py
    python3 scripts/test_storybook_manual_acceptance_complete_next.py
    python3 scripts/test_storybook_manual_acceptance_mark_approved.py
    python3 scripts/test_storybook_manual_acceptance_approve.py
    python3 scripts/test_storybook_manual_acceptance_smoke.py
    python3 scripts/test_storybook_manual_acceptance_final_gate.py
    python3 scripts/test_storybook_interaction_pending_only.py
    python3 scripts/test_verify_release_target.py
    python3 scripts/storybook_native_window_probe.py --self-test
    python3 scripts/assert-strict-coverage-json.py --self-test
    python3 scripts/assert-strict-coverage-lcov.py --self-test
    python3 scripts/coverage/image-runtime-id.py --self-test
    python3 scripts/assert-kuc-release-readiness.py --self-test
    python3 scripts/assert-kuc-release-readiness.py
    python3 scripts/assert-storybook-consumer-contract.py --self-test
    python3 scripts/assert-storybook-consumer-contract.py
    python3 scripts/assert-kuc-guardrails.py
    just raster-host-contract
    python3 scripts/assert-root-plan-task-drift.py

# Backward-compatible alias for older local workflows
kuw-guardrails: kuc-guardrails

# Verify the registry-safe raster host compiles without GUI runtime dependencies.
raster-host-contract:
    {{CARGO}} check -p katana-ui-core --no-default-features --features raster-host --locked
    CARGO="{{CARGO}}" bash scripts/assert-core-dependency-boundary.sh
    bash scripts/assert-core-public-api-neutral.sh

# Install repository-local git hooks
install-hooks:
    bash scripts/install-git-hooks.sh

# Run hook policy validation fixtures.
hook:
    python3 scripts/test_hook_policy.py

# Run Storybook page structure checks
storybook-ast-lint:
    python3 scripts/assert-storybook-page-layout.py

# Audit required Storybook pages are reflected into page-specific surfaces.
storybook-reflection-audit:
    python3 scripts/assert-storybook-reflection-audit.py --strict

# Reject direct overlay lifecycle calls outside the shared guard.
overlay-lifecycle-lint:
    bash scripts/assert-overlay-lifecycle.sh

# Check MenuButton placement and close behavior contracts.
menu-button-contract:
    bash scripts/assert-menu-button-contract.sh

# Run workspace tests
unit-test:
    {{CARGO}} test {{KUC_WORKSPACE_PACKAGES}} --all-targets --all-features --locked

# Run integration tests for generic Rust app consumption.
integration-test: consumer-app-contract

# Run end-to-end headless Storybook scenarios against real component behavior.
e2e-test:
    bash scripts/storybook-requirement-gate.sh

# Run smoke tests for launch and interactive Storybook paths.
smoke-test: storybook-smoke storybook-interaction-smoke

# Alias used by KML-style workflows
test: unit-test

# Run coverage as a release confidence gate
coverage: fmt-check ast-lint
    bash scripts/coverage/prepare-ci-storage.sh "{{REPO_ROOT}}"
    just coverage-container

# Rerun the full strict suite while reusing unchanged coverage build artifacts during iteration.
coverage-iterate: fmt-check ast-lint
    just coverage-container-iterate

# Add one focused adapter target to a complete full-workspace profile during iteration.
# This never replaces the clean full-workspace coverage run in release-check.
coverage-adapter-supplement test_target test_filter: fmt-check ast-lint
    KUC_COVERAGE_SUPPLEMENT_TARGET={{quote(test_target)}} KUC_COVERAGE_SUPPLEMENT_FILTER={{quote(test_filter)}} just coverage-container-adapter-supplement

# Run the Linux/Xvfb coverage implementation directly
coverage-linux:
    CARGO="{{CARGO}}" bash scripts/run-strict-coverage.sh

# Run the Linux/Xvfb iteration path without discarding unchanged coverage build artifacts.
coverage-linux-iterate:
    KUC_COVERAGE_REUSE=1 CARGO="{{CARGO}}" bash scripts/run-strict-coverage.sh

# Supplement an unchanged full-workspace profile with one focused adapter test target on Linux.
coverage-linux-adapter-supplement test_target test_filter:
    KUC_COVERAGE_SUPPLEMENT_TARGET={{quote(test_target)}} KUC_COVERAGE_SUPPLEMENT_FILTER={{quote(test_filter)}} KUC_COVERAGE_REUSE=1 CARGO="{{CARGO}}" bash scripts/run-strict-coverage.sh

# Run strict Linux/Xvfb coverage from macOS or Windows without opening a window
coverage-container:
    just _coverage-container-run 0

# Run the containerized iteration path while keeping the final clean gate separate.
coverage-container-iterate:
    just _coverage-container-run 1

# Supplement a complete full-workspace profile without rerunning unrelated tests.
coverage-container-adapter-supplement:
    just _coverage-container-run 1

_coverage-container-run reuse:
    docker build --tag "{{COVERAGE_IMAGE}}" --file scripts/coverage/Dockerfile scripts/coverage
    bash scripts/coverage/run-container.sh "{{COVERAGE_IMAGE}}" "{{REPO_ROOT}}" "{{COVERAGE_BUILD_JOBS}}" "{{COVERAGE_TEST_THREADS}}" "{{reuse}}"

# Run the local quality gate
check: fmt-check ast-lint check-types lint unit-test kuc-guardrails overlay-lifecycle-lint menu-button-contract
    @echo "checks passed"

# Sweep old build artifacts locally (older than 7 days)
sweep:
    @{{CARGO}} sweep --time 7 || true

# Remove build artifacts
clean: sweep
    {{CARGO}} clean

# Update dependency crates to latest compatible versions
update-safe:
    {{RTK_CMD}}cargo update

# Upgrade all dependency requirements, then update Cargo.lock
update:
    {{RTK_CMD}}cargo upgrade -i
    {{RTK_CMD}}cargo update

# Run the visible KUC Storybook panel window
storybook:
    {{CARGO}} rustc --release -p katana-ui-core-storybook --bin katana-ui-core-storybook --locked -- -D warnings
    {{CARGO}} run --release -p katana-ui-core-storybook --bin katana-ui-core-storybook --locked -- --open-window 0

# Print the KUC Storybook validation summary without opening a window
storybook-summary:
    {{CARGO}} rustc -p katana-ui-core-storybook --bin katana-ui-core-storybook --locked -- -D warnings
    {{CARGO}} run -p katana-ui-core-storybook --bin katana-ui-core-storybook --locked

# Run the visible KUC Storybook panel plus modal native window
storybook-modal:
    {{CARGO}} rustc --release -p katana-ui-core-storybook --bin katana-ui-core-storybook --locked -- -D warnings
    {{CARGO}} run --release -p katana-ui-core-storybook --bin katana-ui-core-storybook --locked -- --open-modal-window 0

# Check that Storybook compiles without running
storybook-check:
    {{CARGO}} rustc -p katana-ui-core-storybook --lib --locked -- -D warnings
    {{CARGO}} rustc -p katana-ui-core-storybook --bin katana-ui-core-storybook --locked -- -D warnings

# Render the KUC Storybook panel to a PNG snapshot.
storybook-visual-snapshot:
    {{CARGO}} rustc --release -p katana-ui-core-storybook --bin katana-ui-core-storybook --locked -- -D warnings
    {{CARGO}} run --release -p katana-ui-core-storybook --bin katana-ui-core-storybook --locked -- --visual-snapshot target/storybook-panel.png

# Check Storybook pages can build without opening visible windows.
storybook-smoke:
    bash scripts/storybook-headless-smoke.sh

# Verify VERSION follows the published release line
release-target-check:
    bash scripts/release/verify-version.sh "{{VERSION}}"
    GH_TOKEN="${GH_TOKEN:-$(gh auth token 2>/dev/null || true)}" python3 scripts/release/verify-release-target.py --target-version "{{VERSION}}" --repo "{{RELEASE_REPO}}"

# Verify KUC v0.1.0 DoD is actually complete before release gates continue.
release-readiness-check: integration-test e2e-test smoke-test
    python3 scripts/assert-kuc-release-readiness.py --self-test
    python3 scripts/assert-kuc-release-readiness.py

# Verify package metadata and dry-run the publishable crate
release-verify: check coverage
    bash scripts/release/verify-version.sh "{{VERSION}}"
    {{CARGO}} package -p katana-ui-core --locked --allow-dirty
    {{CARGO}} publish -p katana-ui-core --dry-run --locked --allow-dirty
    bash scripts/release/verify-core-release-scope.sh "{{VERSION}}"

# Verify release branch readiness before merging
release-check: release-target-check fmt-check ast-lint release-readiness-check release-verify
    bash scripts/release/assert-crate-not-published.sh "{{VERSION}}"

# Show recent Release workflow runs
release-status:
    gh run list --repo {{RELEASE_REPO}} --workflow Release --limit 5

# After public release, switch to the default branch and remove only merged release branches.
release-local-cleanup:
    python3 scripts/release/cleanup-release-branches.py --version "{{VERSION}}" --repo "{{RELEASE_REPO}}"

# Check Storybook overlay/action pages in an opened state, not only initial mount.
storybook-interaction-smoke:
    bash scripts/storybook-interaction-smoke.sh

# Drive the visible Storybook native window with OS input and write per-page traces.
storybook-native-window-probe:
    python3 scripts/storybook_native_window_probe.py

# Drive native window acceptance scenarios for pages still blocked from release readiness.
storybook-native-window-matrix:
    python3 scripts/storybook_native_window_probe.py --matrix --output-dir target/manual-ui-probe/native-matrix-release

# Check manual acceptance smoke commands for pages still waiting on user confirmation.
storybook-manual-acceptance-smoke:
    {{CARGO}} run --release -p katana-ui-core-storybook --bin katana-ui-core-storybook --locked -- --headless-interaction-audit
    PYTHONPATH=scripts python3 scripts/storybook_manual_acceptance_smoke.py

# Print manual acceptance checklist for pages still waiting on user confirmation.
storybook-manual-acceptance-review:
    PYTHONPATH=scripts python3 scripts/storybook_manual_acceptance_review.py

# Print machine-readable manual acceptance progress status.
storybook-manual-acceptance-status:
    PYTHONPATH=scripts python3 scripts/storybook_manual_acceptance_status.py

# Print only the next manual acceptance page in dependency order.
storybook-manual-acceptance-next:
    PYTHONPATH=scripts python3 scripts/storybook_manual_acceptance_next.py

# Open only the next manual acceptance page in dependency order.
storybook-manual-acceptance-next-open:
    PYTHONPATH=scripts python3 scripts/storybook_manual_acceptance_next.py --open

# Create an unapproved manual acceptance log template from the pending manifest.
storybook-manual-acceptance-approval-template:
    PYTHONPATH=scripts python3 scripts/storybook_manual_acceptance_approval_template.py

# Complete only the next manual acceptance page after user OK.
storybook-manual-acceptance-complete-next approved_by approved_at:
    PYTHONPATH=scripts python3 scripts/storybook_manual_acceptance_complete_next.py --approved-by "{{approved_by}}" --approved-at "{{approved_at}}"

# Mark one manually confirmed Storybook page as approved in the approval log.
storybook-manual-acceptance-mark-approved page approved_by approved_at:
    PYTHONPATH=scripts python3 scripts/storybook_manual_acceptance_mark_approved.py --page "{{page}}" --approved-by "{{approved_by}}" --approved-at "{{approved_at}}"

# Apply one approved Storybook page to the manifest and ledger.
storybook-manual-acceptance-approve page:
    PYTHONPATH=scripts python3 scripts/storybook_manual_acceptance_approve.py --page "{{page}}"

# Fail until every manual acceptance pending gap has been approved and removed.
storybook-manual-acceptance-final-gate:
    PYTHONPATH=scripts python3 scripts/storybook_manual_acceptance_final_gate.py

# Final KUC Storybook DoD gate after user manual acceptance has been applied.
storybook-kuc-dod-final: storybook-manual-acceptance-final-gate storybook-interaction-smoke

# Check that interaction smoke has no blocking drift; user-confirmation pending gaps are non-blocking.
storybook-interaction-pending-only:
    {{CARGO}} run --release -p katana-ui-core-storybook --bin katana-ui-core-storybook --locked -- --headless-interaction-audit
    PYTHONPATH=scripts python3 scripts/storybook_interaction_pending_only.py

# Check Storybook pages against requirement scenarios, not only crash-free launch.
storybook-requirement-gate:
    bash scripts/storybook-requirement-gate.sh

# Run all KUC Rust tests used by the Storybook regression gate.
cargo-test:
    {{CARGO}} test {{KUC_WORKSPACE_PACKAGES}} --all-targets --locked

# Run the full Storybook regression gate used before publishing.
storybook-regression: cargo-test storybook-check ast-lint kuc-guardrails overlay-lifecycle-lint menu-button-contract storybook-smoke storybook-manual-acceptance-smoke storybook-interaction-pending-only storybook-interaction-smoke storybook-requirement-gate
