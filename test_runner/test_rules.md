# Tous
cargo run --manifest-path test_runner/Cargo.toml --

# VM
cargo run --manifest-path test_runner/Cargo.toml -- --filter vm

# GC
cargo run --manifest-path test_runner/Cargo.toml -- --filter gc

# Classes
cargo run --manifest-path test_runner/Cargo.toml -- --filter classes

# Modules
cargo run --manifest-path test_runner/Cargo.toml -- --filter modules

# Regression
cargo run --manifest-path test_runner/Cargo.toml -- --filter regression

# Tests errors
cargo run --manifest-path test_runner/Cargo.toml -- --errors

# Display lists
cargo run --manifest-path test_runner/Cargo.toml -- --list

# Update expected values ​​after a deliberate modification 
cargo run --manifest-path test_runner/Cargo.toml -- --bless