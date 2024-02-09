#!/bin/bash
# Copyright © 2024 Stephan Kunz

session="workstation"

tmux new-session -d -s $session

window=0
tmux rename-window -t $session:$window 'workstation'

tmux attach-session -t $session
