#!/bin/bash
cargo build --release
mkdir output


for filename in ./fixtures/*.html; do
  echo "Rendering $filename"
  base_file_name=$(basename "$filename" .html)

  cd target/release
  ./moon render --once --html=../../fixtures/$base_file_name.html --size=500x300 --output=../../output/$base_file_name.png
  cd ../../
done
