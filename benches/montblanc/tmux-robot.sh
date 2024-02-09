#!/bin/bash
# Copyright © 2024 Stephan Kunz
# can be called with option `--release` to run release builds

session="robot"

tmux new-session -d -s $session

window=0
tmux rename-window -t $session:$window 'robot'

# split window into 16 panes according to following schema
#
#  +-------------+-------------+-------------+-------------+
#  |             |             |             |             |
#  |      0      |      4      |      8      |     12      |
#  |             |             |             |             |
#  +-------------+-------------+-------------+-------------+
#  |             |             |             |             |
#  |      1      |      5      |      9      |     13      |
#  |             |             |             |             |
#  +-------------+-------------+-------------+-------------+
#  |             |             |             |             |
#  |      2      |      6      |     10      |     14      |
#  |             |             |             |             |
#  +-------------+-------------+-------------+-------------+
#  |             |             |             |             |
#  |      3      |      7      |     11      |     15      |
#  |             |             |             |             |
#  +-------------+-------------+-------------+-------------+

tmux split-pane -h -p 80
tmux split-pane -h -p 67
tmux split-pane -h -p 50

tmux select-pane -t 0
tmux split-pane -v -p 80
tmux split-pane -v -p 67
tmux split-pane -v -p 50

tmux select-pane -t 4
tmux split-pane -v -p 80
tmux split-pane -v -p 67
tmux split-pane -v -p 50

tmux select-pane -t 8
tmux split-pane -v -p 80
tmux split-pane -v -p 67
tmux split-pane -v -p 50

tmux select-pane -t 12
tmux split-pane -v -p 80
tmux split-pane -v -p 67
tmux split-pane -v -p 50

# start an example in each pane but panes 0, 15
# end with active pane 0
tmux select-pane -t 1
tmux send-keys "cargo run --bin cordoba $1" C-m
tmux select-pane -t 2
tmux send-keys "cargo run --bin lyon $1" C-m
tmux select-pane -t 3
tmux send-keys "cargo run --bin freeport $1" C-m
tmux select-pane -t 4
tmux send-keys "cargo run --bin medellin $1" C-m
tmux select-pane -t 5
tmux send-keys "cargo run --bin portsmouth $1" C-m
tmux select-pane -t 6
tmux send-keys "cargo run --bin hamburg $1" C-m
tmux select-pane -t 7
tmux send-keys "cargo run --bin delhi $1" C-m
tmux select-pane -t 8
tmux send-keys "cargo run --bin taipei $1" C-m
tmux select-pane -t 9
tmux send-keys "cargo run --bin osaka $1" C-m
tmux select-pane -t 10
tmux send-keys "cargo run --bin hebron $1" C-m
tmux select-pane -t 11
tmux send-keys "cargo run --bin kingston $1" C-m
tmux select-pane -t 12
tmux send-keys "cargo run --bin tripoli $1" C-m
tmux select-pane -t 13
tmux send-keys "cargo run --bin mandalay $1" C-m
tmux select-pane -t 14
tmux send-keys "cargo run --bin ponce $1" C-m

tmux select-pane -t 0

tmux attach-session -t $session
