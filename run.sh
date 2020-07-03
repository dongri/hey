#!/bin/bash

cargo build --release
pkill hey
./target/release/hey >> release.log &
