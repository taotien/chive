export RUST_LOG := "chive=trace"

init:
    cargo run -- init -p testing -y

run:
    cargo run -- run -p testing mnt
