export RUST_LOG := "trace"

init:
    cargo run -- init -p /home/tao/videos/anime/inbox -y

run:
    cargo run -- run -p /home/tao/videos/anime/inbox mnt
