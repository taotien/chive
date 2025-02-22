set shell := ["nu", "-c"]

export RUST_LOG := "chive=trace"

run:
    try {umount mnt}; cargo run -- run -p testing mnt

init:
    cargo run -- init -p testing -y

setup:
    #!/usr/bin/env nu
    try { 
        mkdir testing mnt 
        touch testing/hello.txt 
    }

clear:
    try {rm testing/*.chive}
