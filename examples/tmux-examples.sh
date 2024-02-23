#!/bin/bash
# Copyright © 2024 Stephan Kunz
# can be called with option `--release` to run release builds

session="examples"

tmux new-session -d -s $session

window=0
tmux rename-window -t $session:$window 'examples'

# split window into 10 panes according to following schema
#
#  +-------------+-------------+-------------+
#  |             |             |             |
#  |      0      |      3      |      7      |
#  |             |             |             |
#  +-------------+-------------+-------------+
#  |             |             |             |
#  |      1      |      4      |      8      |
#  |             |             |             |
#  +-------------+-------------+-------------+
#  |             |      5      |             |
#  |      2      +-------------+      9      |
#  |             |      6      |             |
#  +-------------+-------------+-------------+

tmux split-pane -h -p 70
tmux split-pane -h -p 50

tmux select-pane -t 0
tmux split-pane -v -p 66
tmux split-pane -v -p 50

tmux select-pane -t 3
tmux split-pane -v -p 50
tmux split-pane -v -p 50
tmux split-pane -v -p 50

tmux select-pane -t 7
tmux split-pane -v -p 66
tmux split-pane -v -p 50

# start an example in each pane but pane 9
# end with active pane 9
# publisher & subscriber
tmux select-pane -t 0
tmux send-keys "cargo run --bin publisher $1" C-m
tmux select-pane -t 1
tmux send-keys "cargo run --bin publisher $1" C-m
tmux select-pane -t 2
tmux send-keys "cargo run --bin subscriber $1" C-m
# query & queryable
tmux select-pane -t 3
tmux send-keys "cargo run --bin query $1" C-m
tmux select-pane -t 4
tmux send-keys "cargo run --bin queryable $1" C-m
# liveliness
tmux select-pane -t 5
tmux send-keys "cargo run --bin liveliness $1" C-m
tmux select-pane -t 6
tmux send-keys "cargo run --bin liveliness $1" C-m
# ping/pong
tmux select-pane -t 7
tmux send-keys "cargo run --bin ping $1" C-m
tmux select-pane -t 8
tmux send-keys "cargo run --bin pong $1" C-m

tmux select-pane -t 9

tmux attach-session -t $session
