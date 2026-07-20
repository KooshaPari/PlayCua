#!/bin/sh
# fake-nvms.sh — emulates `nvms run --config <path>` for hermetic tests.
# Prints a marker, sleeps a long time to allow shutdown tests to run, then exits.
echo "FAKE-NVMS-ALIVE rev-1"
sleep 30
echo "FAKE-NVMS-DONE"
exit 0
