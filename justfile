# Verfügbare Rezepte anzeigen
default:
    @just --list

# Volles Programm, das auch die CI ausführt
check: fmt-check lint test

# Schnelle Prüfung für den Pre-Commit-Hook
quick: fmt-check lint

# Hooks aktivieren, einmal pro Klon nötig
setup:
    git config core.hooksPath .githooks

fmt:
    cargo fmt

fmt-check:
    cargo fmt --check

lint:
    cargo clippy --all-targets -- -D warnings

test:
    cargo test

doc:
    cargo doc --open