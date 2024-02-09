#!/bin/bash
# Copyright © 2024 Stephan Kunz

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
tmux split-pane -v -p 70
tmux split-pane -v -p 50

tmux select-pane -t 3
tmux split-pane -v -p 56
tmux split-pane -v -p 30
tmux split-pane -v

tmux select-pane -t 7
tmux split-pane -v -p 70
tmux split-pane -v -p 50

# start an example in each pane but pane 9
# end with active pane 9
tmux select-pane -t 0
tmux send-keys "cargo run --bin publisher" C-m

tmux select-pane -t 1
tmux send-keys "cargo run --bin publisher" C-m

tmux select-pane -t 2
tmux send-keys "cargo run --bin subscriber" C-m

tmux select-pane -t 3
tmux send-keys "cargo run --bin query" C-m

tmux select-pane -t 4
tmux send-keys "cargo run --bin queryable" C-m

tmux select-pane -t 5
tmux send-keys "cargo run --bin liveliness" C-m

tmux select-pane -t 6
tmux send-keys "cargo run --bin liveliness" C-m

tmux select-pane -t 7
tmux send-keys "cargo run --bin ping" C-m

tmux select-pane -t 8
tmux send-keys "cargo run --bin pong" C-m


tmux select-pane -t 9

tmux attach-session -t $session
