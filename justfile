set shell := ["nu", "-c"]

export RUST_LOG := "chive=trace"

init:
    cargo run -- init -p testing -y

run:
    try {umount mnt}; cargo run -- run -p testing mnt
