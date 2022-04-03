#!/bin/bash


set -o errexit
set -o xtrace


gcc -o simple.o -c simple.c
gcc -o simple simple.o
