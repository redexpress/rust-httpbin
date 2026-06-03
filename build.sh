#!/usr/bin/env bash
# Local CI mirror for axum-httpbin.
#
# Mirrors .github/workflows/test.yml. Defaults to a slightly stricter
# local run (full test suite including integration tests); use `ci` for
# exact GitHub Actions parity.

set -euo pipefail

cd "$(dirname "$0")"
export CARGO_TERM_COLOR=always

step() { printf '\n\033[1;34m== %s ==\033[0m\n' "$1"; }
ok()   { printf '\033[1;32m✓ %s\033[0m\n' "$1"; }

do_fmt() {
    step "cargo fmt --check"
    cargo fmt --check
    ok "fmt"
}

do_clippy() {
    step "cargo clippy -- -D warnings"
    cargo clippy -- -D warnings
    ok "clippy"
}

do_test_lib() {
    step "cargo test --lib (CI parity)"
    cargo test --lib
    ok "test --lib"
}

do_test() {
    step "cargo test (full suite, also runs integration tests)"
    cargo test
    ok "test"
}

do_build() {
    step "cargo build --release"
    cargo build --release
    ok "build"
}

do_ci()  { do_fmt; do_clippy; do_test_lib; do_build; }
do_all() { do_fmt; do_clippy; do_test; do_build; }

usage() {
    cat <<EOF
Usage: $0 [command...]

Commands:
  fmt         cargo fmt --check
  clippy      cargo clippy -- -D warnings
  test        cargo test (full suite, including integration tests)
  test-lib    cargo test --lib (matches CI exactly)
  build       cargo build --release
  ci          fmt + clippy + test-lib + build  (exact GitHub Actions parity)
  all         fmt + clippy + test + build       (default — slightly stricter)
  -h | --help show this help

If no command is given, \`all\` runs.
EOF
}

main() {
    if [ $# -eq 0 ]; then
        do_all
    else
        for cmd in "$@"; do
            case "$cmd" in
                fmt)      do_fmt ;;
                clippy)   do_clippy ;;
                test)     do_test ;;
                test-lib) do_test_lib ;;
                build)    do_build ;;
                ci)       do_ci ;;
                all)      do_all ;;
                -h|--help) usage; exit 0 ;;
                *) printf 'unknown command: %s\n' "$cmd" >&2
                   usage
                   exit 2 ;;
            esac
        done
    fi

    printf '\n\033[1;32m== all checks passed ==\033[0m\n'
}

main "$@"
