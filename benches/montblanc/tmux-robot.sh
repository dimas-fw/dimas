#!/bin/bash
# Copyright © 2024 Stephan Kunz

session="robot"

tmux new-session -d -s $session

window=0
tmux rename-window -t $session:$window 'robot'

tmux attach-session -t $session
