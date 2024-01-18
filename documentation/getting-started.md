# Getting Started

## Prerequisites

Make sure you have all the dependencies installed:

**Debian / Ubuntu**

```
sudo apt install build-essential cmake pkg-config libssl-dev mold
```

> Optional: [`cargo-make`](https://github.com/sagiegurari/cargo-make) for running predefined building & running tasks.

## Run with UI

By default Moon will start with a iced-rs based UI, so to run the browser with the UI, execute the command:

```
cargo run
```

## Run without UI

You can run Moon without the UI & export the rendering into an image by running:

```
cargo run -- render --once --html=<path_to_html> --size=<width>x<height> --output=<path_to_image>.png
```

For example:

```
cargo run -- render --once --html=fixtures/test.html --size=900x400 --output=image.png
```

Or if you have `cargo-make` installed, you can quickly render HTML files in the [`fixtures/`](../fixtures/) folder by calling:

```
cargo make try <file_name_without_dot_html>
```
