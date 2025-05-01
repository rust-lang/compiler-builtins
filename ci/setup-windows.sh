#!/bin/bash

set -eux

ls /
ls /c/msys64

export PATH="/c/msys64:$PATH"

pacman install m4

mv -h
