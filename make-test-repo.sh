#!/usr/bin/env bash

set -Eeuo pipefail

cd "$(dirname "$0")"

rm -rf test-repo || true

# The commit graph currently being used for testing purposes:
#
# * K (k)
# * H
# |\
# * | E
# | | * M (main, m)
# | | |\
# | | | * J
# | | * | L
# | | * | I
# | |/| |
# | | |/
# | | * G
# | * | F
# |/| |
# | |/
# | * D
# * | C
# |/
# * B
# * A

git init --initial-branch k test-repo
cd test-repo
touch a.txt && git add a.txt && git commit -m "A"
touch b.txt && git add b.txt && git commit -m "B"
git branch j
touch c.txt && git add c.txt && git commit -m "C"
git switch j
touch d.txt && git add d.txt && git commit -m "D"
git switch k
git branch m
touch e.txt && git add e.txt && git commit -m "E"
git switch m
git merge j -m "F" && touch f.txt && git add f.txt && git commit --amend --no-edit
git switch j
touch g.txt && git add g.txt && git commit -m "G"
git switch k
git merge m -m "H" && touch h.txt && git add h.txt && git commit --amend --no-edit
git switch m
git merge j -m "I" && touch i.txt && git add i.txt && git commit --amend --no-edit
git switch j
touch j.txt && git add j.txt && git commit -m "J"
git switch k
touch k.txt && git add k.txt && git commit -m "K"
git switch m
touch l.txt && git add l.txt && git commit -m "L"
git merge j -m "M" && touch m.txt && git add m.txt && git commit --amend --no-edit
git branch -d j
git switch -c main
