#!/bin/bash
cargo build --release
mkdir output


for filename in ./fixtures/*.html; do
  echo "Rendering $filename"
  base_file_name=$(basename "$filename" .html)

  target/release/moon render --once --html=fixtures/$base_file_name.html --size=1200x600 --output=output/$base_file_name.png
done
