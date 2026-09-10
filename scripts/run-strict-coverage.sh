#!/usr/bin/env bash
set -euo pipefail

if [[ "$(uname -s)" != "Linux" ]]; then
  echo "strict coverage requires Linux with Xvfb" >&2
  exit 1
fi

if ! command -v Xvfb >/dev/null; then
  echo "Xvfb is required for strict native-window coverage" >&2
  exit 1
fi

if [[ -n "${CARGO:-}" ]]; then
  read -r -a cargo_command <<<"${CARGO}"
elif command -v rtk >/dev/null; then
  cargo_command=(rtk cargo)
else
  cargo_command=(cargo)
fi

run_cargo() {
  "${cargo_command[@]}" "$@"
}

run_cargo_raw() {
  if [[ "${cargo_command[0]}" == "rtk" ]]; then
    rtk proxy "${cargo_command[@]:1}" "$@"
  else
    "${cargo_command[@]}" "$@"
  fi
}

display_number=99
while [[ -e "/tmp/.X${display_number}-lock" || -S "/tmp/.X11-unix/X${display_number}" ]]; do
  display_number=$((display_number + 1))
done
xvfb_log="${CARGO_TARGET_DIR:-target}/kuc-xvfb.log"
mkdir -p "$(dirname "${xvfb_log}")"
Xvfb ":${display_number}" -screen 0 1600x1200x24 -nolisten tcp -ac >"${xvfb_log}" 2>&1 &
xvfb_pid=$!

cleanup_xvfb() {
  kill "${xvfb_pid}" 2>/dev/null || true
  wait "${xvfb_pid}" 2>/dev/null || true
}

finalize_coverage_process() {
  local exit_status=$?
  trap - EXIT
  if [[ "${coverage_transaction_active:-0}" == "1" ]]; then
    write_coverage_profile_state invalid
    write_coverage_strict_state invalid
  fi
  cleanup_xvfb
  exit "${exit_status}"
}
trap finalize_coverage_process EXIT

for _ in {1..100}; do
  if [[ -S "/tmp/.X11-unix/X${display_number}" ]]; then
    break
  fi
  if ! kill -0 "${xvfb_pid}" 2>/dev/null; then
    cat "${xvfb_log}" >&2
    exit 1
  fi
  sleep 0.1
done
if [[ ! -S "/tmp/.X11-unix/X${display_number}" ]]; then
  echo "Xvfb did not create its X11 socket" >&2
  cat "${xvfb_log}" >&2
  exit 1
fi
export DISPLAY=":${display_number}"
export XDG_RUNTIME_DIR="${CARGO_TARGET_DIR:-target}/kuc-xdg-runtime"
mkdir -p "${XDG_RUNTIME_DIR}"
chmod 700 "${XDG_RUNTIME_DIR}"
export KUC_STORYBOOK_MOUSE_TRACE="${CARGO_TARGET_DIR:-target}/kuc-storybook-mouse-trace.jsonl"
# LLVM更新で実行済みgeneric関数が未到達の最適化instanceとして集計されないようにする。
export CARGO_PROFILE_TEST_OPT_LEVEL=0

coverage_storage_dir="${CARGO_TARGET_DIR:-target}"
coverage_target_dir="${coverage_storage_dir}/llvm-cov-target"
coverage_reuse="${KUC_COVERAGE_REUSE:-0}"
coverage_test_threads="${COVERAGE_TEST_THREADS:-8}"
coverage_supplement_target="${KUC_COVERAGE_SUPPLEMENT_TARGET:-lib}"
coverage_supplement_filter="${KUC_COVERAGE_SUPPLEMENT_FILTER:-}"
coverage_runtime="${KUC_COVERAGE_RUNTIME:-native}"
coverage_image_id="${KUC_COVERAGE_IMAGE_ID:-}"
coverage_profile_path="${coverage_storage_dir}/kuc-workspace-coverage-profile-v3.sha256"
coverage_strict_state_path="${coverage_profile_path}.strict-state"
coverage_report_path="${coverage_storage_dir}/kuc-workspace-coverage-summary.json"
coverage_lcov_path="${coverage_storage_dir}/kuc-workspace-coverage.lcov"
coverage_started_at="${SECONDS}"
coverage_transaction_active=0
coverage_target_ownership_verified=0
readonly cargo_cache_tag_signature="Signature: 8a477f597d28d172789f06886806bc55"
readonly coverage_target_owner_signature="katana-ui-core strict coverage target v1"
readonly coverage_target_owner_filename=".kuc-strict-coverage-target-v1"

ensure_coverage_target_cache_dir() {
  if [[ -L "${coverage_target_dir}" ]]; then
    echo "strict coverage target must not be a symlink" >&2
    exit 1
  fi
  mkdir -p "${coverage_target_dir}"

  local cache_tag="${coverage_target_dir}/CACHEDIR.TAG"
  local owner_marker="${coverage_target_dir}/${coverage_target_owner_filename}"
  if [[ -e "${owner_marker}" || -L "${owner_marker}" ]]; then
    if [[ -L "${owner_marker}" || ! -f "${owner_marker}" \
      || "$(<"${owner_marker}")" != "${coverage_target_owner_signature}" ]]; then
      echo "strict coverage target has an invalid KUC ownership marker" >&2
      exit 1
    fi
  elif [[ -n "$(find "${coverage_target_dir}" -mindepth 1 -maxdepth 1 -print -quit)" ]]; then
    echo "strict coverage target is unowned and not empty: ${coverage_target_dir}" >&2
    exit 1
  else
    printf '%s\n' "${coverage_target_owner_signature}" >"${owner_marker}"
  fi
  coverage_target_ownership_verified=1

  if [[ -e "${cache_tag}" || -L "${cache_tag}" ]]; then
    if [[ -L "${cache_tag}" || ! -f "${cache_tag}" \
      || "$(head -n 1 "${cache_tag}")" != "${cargo_cache_tag_signature}" ]]; then
      echo "strict coverage target has an invalid Cargo cache marker" >&2
      exit 1
    fi
    return
  fi

  # WHY: KUC が新規または空の専用 target の所有権を先に記録してから、
  # cargo-llvm-cov の cleanup を許可する Cargo cache marker を作る。
  printf '%s\n%s\n%s\n' \
    "${cargo_cache_tag_signature}" \
    "# This file is a cache directory tag created by cargo." \
    "# For information about cache directory tags see https://bford.info/cachedir/" \
    >"${cache_tag}"
}

restore_coverage_target_owner_marker_after_cleanup() {
  if [[ "${coverage_target_ownership_verified}" != "1" ]]; then
    echo "strict coverage target ownership was not verified before cleanup" >&2
    exit 1
  fi
  if [[ -L "${coverage_target_dir}" ]]; then
    echo "strict coverage target must not be a symlink" >&2
    exit 1
  fi
  mkdir -p "${coverage_target_dir}"

  local owner_marker="${coverage_target_dir}/${coverage_target_owner_filename}"
  if [[ -e "${owner_marker}" || -L "${owner_marker}" ]]; then
    if [[ -L "${owner_marker}" || ! -f "${owner_marker}" \
      || "$(<"${owner_marker}")" != "${coverage_target_owner_signature}" ]]; then
      echo "strict coverage target has an invalid KUC ownership marker" >&2
      exit 1
    fi
    return
  fi

  # WHY: cargo clean は target 内の非Cargoファイルも削除し得る。cleanup 前に
  # 検証済みの専用 target に限り、次回の unowned 判定を避ける marker を復元する。
  printf '%s\n' "${coverage_target_owner_signature}" >"${owner_marker}"
}

native_coverage_runtime_id() {
  {
    printf '%s\n' native-linux
    uname -a
    sha256sum "$(command -v Xvfb)"
    if [[ -f /etc/os-release ]]; then
      sha256sum /etc/os-release
    fi
    ldd --version 2>&1 | sed -n '1p'
  } | sha256sum | awk '{ print $1 }'
}

case "${coverage_runtime}" in
  container)
    if [[ ! "${coverage_image_id}" =~ ^runtime-v1:sha256:[0-9a-f]{64}$ ]]; then
      echo "container coverage requires a validated runtime image identity" >&2
      exit 1
    fi
    ;;
  native)
    if [[ -n "${coverage_image_id}" ]]; then
      echo "native coverage does not accept KUC_COVERAGE_IMAGE_ID" >&2
      exit 1
    fi
    coverage_image_id="native-linux:$(native_coverage_runtime_id)"
    ;;
  *)
    echo "KUC_COVERAGE_RUNTIME must be native or container" >&2
    exit 1
    ;;
esac

coverage_production_digest() {
  find \
    crates/katana-ui-core/src \
    crates/katana-ui-core-storybook/src \
    examples/kuc-consumer-app/src \
    -type f \
    -name '*.rs' \
    ! -name 'tests.rs' \
    ! -name '*_tests.rs' \
    ! -path '*/tests/*' \
    ! -path '*_tests/*' \
    -print0 \
    | sort -z \
    | xargs -0 -r sha256sum \
    | sha256sum \
    | awk '{ print $1 }'
}

coverage_profile_signature() {
  {
    printf '%s\n' \
      'version=3' \
      'scope=full-workspace' \
      'packages=katana-ui-core,katana-ui-core-storybook,kuc-consumer-app' \
      'targets=all' \
      'features=all' \
      'profile-test-opt-level=0'
    printf 'runtime-image-id=%s\n' "${coverage_image_id}"
    printf 'production-digest=%s\n' "$(coverage_production_digest)"
    sha256sum \
      Cargo.toml \
      Cargo.lock \
      Justfile \
      scripts/run-strict-coverage.sh \
      scripts/assert-strict-coverage-json.py \
      scripts/assert-strict-coverage-lcov.py \
      scripts/coverage/run-test-binaries.py \
      scripts/coverage/image-runtime-id.py \
      scripts/coverage/run-container.sh \
      scripts/coverage/run-in-container.sh \
      scripts/coverage/Dockerfile \
      crates/katana-ui-core/Cargo.toml \
      crates/katana-ui-core-storybook/Cargo.toml \
      examples/kuc-consumer-app/Cargo.toml
    while IFS= read -r -d '' optional_input; do
      sha256sum "${optional_input}"
    done < <(
      {
        if [[ -d .cargo ]]; then
          find .cargo -maxdepth 1 -type f \( -name config -o -name config.toml \) -print0
        fi
        find . -maxdepth 1 -type f \
          \( -name rust-toolchain -o -name rust-toolchain.toml \) -print0
      } | sort -z
    )
    rustc -vV
    run_cargo llvm-cov --version
  } | sha256sum | awk '{ print $1 }'
}

write_coverage_state() {
  local path="$1"
  local value="$2"
  local temporary_path="${path}.tmp.${BASHPID}"
  printf '%s\n' "${value}" >"${temporary_path}"
  mv "${temporary_path}" "${path}"
}

write_coverage_profile_state() {
  local value="$1"
  write_coverage_state "${coverage_profile_path}" "${value}"
}

write_coverage_strict_state() {
  local value="$1"
  write_coverage_state "${coverage_strict_state_path}" "${value}"
}

invalidate_coverage_profile() {
  coverage_transaction_active=1
  write_coverage_profile_state in-progress
  write_coverage_strict_state in-progress
}

coverage_packages=(
  -p katana-ui-core
  -p katana-ui-core-storybook
  -p kuc-consumer-app
)
if [[ -n "${coverage_supplement_filter}" && "${coverage_reuse}" != "1" ]]; then
  echo "KUC_COVERAGE_SUPPLEMENT_FILTER requires reuse enabled" >&2
  exit 1
fi
if [[ -z "${coverage_supplement_filter}" && "${coverage_supplement_target}" != "lib" ]]; then
  echo "KUC_COVERAGE_SUPPLEMENT_TARGET requires KUC_COVERAGE_SUPPLEMENT_FILTER" >&2
  exit 1
fi
case "${coverage_supplement_target}" in
  lib)
    coverage_supplement_target_args=(--lib)
    ;;
  *[!a-zA-Z0-9_-]* | "")
    echo "KUC_COVERAGE_SUPPLEMENT_TARGET must be lib or an integration test target" >&2
    exit 1
    ;;
  *)
    coverage_supplement_target_args=(--test "${coverage_supplement_target}")
    ;;
esac
case "${coverage_reuse}" in
  0)
    coverage_mode="clean"
    coverage_cleanup_mode="clean"
    pending_profile_signature="$(coverage_profile_signature)"
    ;;
  1)
    if [[ -n "${coverage_supplement_filter}" ]]; then
      coverage_mode="supplement"
      if [[ ! -f "${coverage_profile_path}" ]]; then
        echo "coverage supplement requires a complete full-workspace profile" >&2
        exit 1
      fi
      current_profile_signature="$(coverage_profile_signature)"
      if [[ "$(<"${coverage_profile_path}")" != "${current_profile_signature}" ]]; then
        echo "coverage profile is incomplete or its production inputs changed; rerun full coverage before supplementing" >&2
        exit 1
      fi
      coverage_cleanup_mode="none"
      pending_profile_signature="${current_profile_signature}"
    else
      pending_profile_signature="$(coverage_profile_signature)"
      if [[ -f "${coverage_profile_path}" \
        && "$(<"${coverage_profile_path}")" == "${pending_profile_signature}" ]]; then
        coverage_mode="reuse"
        coverage_cleanup_mode="profraw"
      else
        coverage_mode="rebuild"
        coverage_cleanup_mode="workspace"
      fi
    fi
    ;;
  *)
    echo "KUC_COVERAGE_REUSE must be 0 or 1" >&2
    exit 1
    ;;
esac
if [[ "${coverage_test_threads}" == "auto" ]]; then
  coverage_test_threads="$(getconf _NPROCESSORS_ONLN 2>/dev/null || true)"
  if [[ ! "${coverage_test_threads}" =~ ^[1-9][0-9]*$ ]]; then
    coverage_test_threads=4
  elif ((coverage_test_threads > 12)); then
    coverage_test_threads=12
  fi
elif [[ ! "${coverage_test_threads}" =~ ^[1-9][0-9]*$ ]]; then
  echo "COVERAGE_TEST_THREADS must be auto or a positive integer" >&2
  exit 1
fi
coverage_parallel_binaries=2
if ((coverage_test_threads < coverage_parallel_binaries)); then
  coverage_parallel_binaries=1
fi
coverage_threads_per_binary="$((coverage_test_threads / coverage_parallel_binaries))"
coverage_min_free_gib="${KUC_COVERAGE_MIN_FREE_GIB:-2}"
if [[ ! "${coverage_min_free_gib}" =~ ^[1-9][0-9]*$ ]]; then
  echo "KUC_COVERAGE_MIN_FREE_GIB must be a positive integer" >&2
  exit 1
fi
coverage_available_kib="$(df -Pk "${coverage_storage_dir}" | awk 'NR == 2 { print $4 }')"
coverage_required_kib="$((coverage_min_free_gib * 1024 * 1024))"
if ((coverage_available_kib < coverage_required_kib)); then
  echo "strict coverage requires at least ${coverage_min_free_gib} GiB free after cleanup" >&2
  exit 1
fi
ensure_coverage_target_cache_dir
invalidate_coverage_profile
export CARGO_TARGET_DIR="${coverage_target_dir}"
eval "$(run_cargo_raw llvm-cov show-env --sh)"
case "${coverage_cleanup_mode}" in
  clean)
    run_cargo clean --target-dir "${coverage_target_dir}"
    run_cargo llvm-cov clean --workspace
    ;;
  profraw)
    run_cargo llvm-cov clean --profraw-only
    ;;
  workspace)
    run_cargo llvm-cov clean --workspace
    ;;
  none)
    ;;
esac
restore_coverage_target_owner_marker_after_cleanup
echo "coverage mode: ${coverage_mode}; scope: full; parallel binaries: ${coverage_parallel_binaries}; test threads per binary: ${coverage_threads_per_binary}"
coverage_test_started_at="${SECONDS}"
if [[ -n "${coverage_supplement_filter}" ]]; then
  run_cargo_raw test \
    -p katana-ui-core \
    "${coverage_supplement_target_args[@]}" \
    --all-features \
    --locked \
    "${coverage_supplement_filter}" -- \
    --include-ignored \
    --test-threads="${coverage_test_threads}"
  run_cargo llvm-cov report --quiet \
    "${coverage_packages[@]}" \
    --json \
    --summary-only \
    --output-path "${coverage_report_path}" \
    --ignore-filename-regex '(^|/)(tests/|[^/]+_tests/|tests\.rs$|[^/]+_tests\.rs$)'
else
  coverage_run_dir="${coverage_target_dir}/kuc-test-run-${BASHPID}"
  coverage_metadata_path="${coverage_run_dir}/metadata.json"
  coverage_artifacts_path="${coverage_run_dir}/artifacts.jsonl"
  mkdir -p "${coverage_run_dir}"
  run_cargo_raw metadata --format-version 1 --no-deps --locked \
    >"${coverage_metadata_path}"
  run_cargo_raw test \
    "${coverage_packages[@]}" \
    --all-targets \
    --all-features \
    --locked \
    --no-run \
    --message-format=json \
    >"${coverage_artifacts_path}"
  python3 scripts/coverage/run-test-binaries.py \
    --artifact-json "${coverage_artifacts_path}" \
    --metadata-json "${coverage_metadata_path}" \
    --logs-dir "${coverage_run_dir}/logs" \
    --max-parallel-binaries "${coverage_parallel_binaries}" \
    --test-threads "${coverage_threads_per_binary}"
  run_cargo llvm-cov report --quiet \
    "${coverage_packages[@]}" \
    --json \
    --summary-only \
    --output-path "${coverage_report_path}" \
    --ignore-filename-regex '(^|/)(tests/|[^/]+_tests/|tests\.rs$|[^/]+_tests\.rs$)'
fi
python3 scripts/assert-strict-coverage-json.py --validate-profile "${coverage_report_path}"
run_cargo llvm-cov report --quiet \
  "${coverage_packages[@]}" \
  --lcov \
  --output-path "${coverage_lcov_path}" \
  --ignore-filename-regex '(^|/)(tests/|[^/]+_tests/|tests\.rs$|[^/]+_tests\.rs$)'
write_coverage_profile_state "${pending_profile_signature}"
coverage_transaction_active=0
if python3 scripts/assert-strict-coverage-lcov.py "${coverage_lcov_path}"; then
  write_coverage_strict_state "passed:${pending_profile_signature}"
else
  write_coverage_strict_state "failed:${pending_profile_signature}"
  # 失敗したCIでも不足行を特定できるよう、同じprofile・scopeの診断をログへ残す。
  run_cargo_raw llvm-cov report --show-missing-lines --quiet \
    "${coverage_packages[@]}" \
    --ignore-filename-regex '(^|/)(tests/|[^/]+_tests/|tests\.rs$|[^/]+_tests\.rs$)'
  echo "coverage test and report elapsed seconds: $((SECONDS - coverage_test_started_at))"
  echo "strict coverage elapsed seconds: $((SECONDS - coverage_started_at))"
  exit 1
fi
echo "coverage test and report elapsed seconds: $((SECONDS - coverage_test_started_at))"
echo "strict coverage elapsed seconds: $((SECONDS - coverage_started_at))"
