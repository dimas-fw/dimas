#!/bin/bash
# Copyright © 2024 Stephan Kunz
# can be called with option `--release` to run release builds

export ZENOH_RUNTIME='(app: (worker_threads: 2),tx: (max_blocking_threads: 1))'

session="examples"

tmux new-session -d -s $session

window=0
tmux rename-window -t $session:$window 'examples'

# split window into 9 panes according to following schema
#
#  +-------------+-------------+-------------+
#  |             |             |             |
#  |      0      |      3      |      6      |
#  |             |             |             |
#  +-------------+-------------+-------------+
#  |             |             |             |
#  |      1      |      4      |      7      |
#  |             |             |             |
#  +-------------+-------------+-------------+
#  |             |             |             |
#  |      2      |      5      |      8      |
#  |             |             |             |
#  +-------------+-------------+-------------+

tmux split-pane -h -p 70
tmux split-pane -h -p 50

tmux select-pane -t 0
tmux split-pane -v -p 66
tmux split-pane -v -p 50

tmux select-pane -t 3
tmux split-pane -v -p 66
tmux split-pane -v -p 50

tmux select-pane -t 6
tmux split-pane -v -p 66
tmux split-pane -v -p 50

# start an example in each pane
# publisher & subscriber
tmux select-pane -t 0
tmux send-keys "cargo run --example publisher $1" C-m
tmux select-pane -t 1
tmux send-keys "cargo run --example subscriber $1" C-m

# query & queryable
tmux select-pane -t 3
tmux send-keys "cargo run --example querier $1" C-m
tmux select-pane -t 4
tmux send-keys "cargo run --example queryable $1" C-m

# ping/pong
#tmux select-pane -t 2
#tmux send-keys "cargo run --example ping $1" C-m
#tmux select-pane -t 5
#tmux send-keys "cargo run --example pong $1" C-m

# observation
tmux select-pane -t 6
tmux send-keys "cargo run --example observer $1" C-m
tmux select-pane -t 7
tmux send-keys "cargo run --example observable $1" C-m

tmux select-pane -t 5

# liveliness
#tmux select-pane -t 8
#tmux send-keys "cargo run --example liveliness $1" C-m


tmux attach-session -t $session
