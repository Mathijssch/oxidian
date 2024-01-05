#!/bin/bash

# Compile your Rust program
cargo build --release

# Run your Rust program in the background
./target/release/main build --dir examples/full/Notebook &

# Get the process ID of the Rust program
PID=$!

# Monitor memory usage using ps command
while true; do
    # Use 'ps' to get the memory usage of the process with PID
    MEM_USAGE=$(ps -p $PID -o rss=)

    # Convert KB to MB
    MEM_USAGE_MB=$((MEM_USAGE / 1024)) 

    # Print memory usage
    echo "Memory usage of process $PID: $MEM_USAGE_MB MB"
    
    # Sleep for some time (adjust as needed)
    sleep 0.1
done

