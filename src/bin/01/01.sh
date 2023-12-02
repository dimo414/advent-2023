#!/bin/bash

digitize() {
  local digits=(zero one two three four five six seven eight nine) args=()
  for i in "${!digits[@]}"; do
    d="${digits[i]}"
    args+=(-e "s/${d}/\0${i}\0/g")
  done
  sed "${args[@]}"
}

extract_exterior_digits() {
  local args=(
    # Strip up to the first digit
    -e 's/^[^0-9]*//'
    # Strip back from the last digit
    -e 's/[^0-9]*$//'
    # Retain just the first and last characters (both digits)
    -e 's/\(.\).*\(.\)/\1\2/'
    # Duplicate a single digit
    -e 's/^.$/\0\0/'
  )
  sed "${args[@]}"
}

sum() { awk '{sum += $1} END {print sum}'; }

input="${1:?input file missing}"
printf 'Part 1:\t'
<"$input" extract_exterior_digits | sum
printf 'Part 2:\t'
<"$input" digitize | extract_exterior_digits | sum
