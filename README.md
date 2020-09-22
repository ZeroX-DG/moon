<p align="center"><img src="./resources/logo.png" /></p>

<h1 align="center">Moon</h1>
<p align="center">A hobby web browser developed from scratch</p>

<p align="center">
  <img src="https://img.shields.io/badge/license-MIT-blue" alt="MIT License" />
  <a href="https://webuild.community">
    <img src="https://raw.githubusercontent.com/webuild-community/badge/master/svg/by.svg" alt="By Vietnamese" />
  </a>
</p>

## What is this?

This is a web browser developed from scratch using Rust. I created this project to practice my Rust skills as well as to learn how the browser works.

To fit with the "make from scratch" spirit, I'll limit the use of external libraries as much as possible.

## Features

Bold text is what I'm working on

- [x] :electric_plug: DOM API
- [x] :memo: HTML Parsing
  - [x] HTML tokenizer
  - [x] **HTML dom tree builder (70% complete)**
- [ ] :triangular_ruler: CSS
  - [ ] **CSSOM API**
  - [ ] CSS tokenizer
  - [ ] CSSOM builder
  - [ ] Selector matching
    - [ ] Element selector
    - [ ] Id selector
    - [ ] Class selector
  - [ ] Box model
  - [ ] Block layout
  - [ ] Inline layout
  - [ ] Flexbox
  - [ ] CSS Grid
- [ ] :art: Rendering
  - [ ] GPU rendering
  - [ ] Font rendering
- [ ] :earth_americas: Networking
  - [ ] URL parsing
  - [ ] DNS resolving
  - [ ] DNS caching
  - [ ] HTTP/HTTPS
- [ ] :framed_picture: Media
  - [ ] :framed_picture: Image rendering
    - [ ] JPG
    - [ ] PNG
    - [ ] GIF
  - [ ] :clapper: Video playing
    - [ ] MP4
    - [ ] WebM
  - [ ] :speaker: Audio playing
    - [ ] MP3
    - [ ] WAV
- [ ] JavaScript

## Blog posts

I write about what I learn from this journey on my blog (order by latest):

### Browser from Scratch: DOM API

One of the main building blocks of the HTML rendering process is the DOM API. Before a browser can render the HTML document, it needs to parse the document content into a tree structure called the DOM tree. In this post, I'll break down my experimentation in building a DOM API with Rust. - [**Read more**][2]

### Browser from Scratch: Introduction

This is the start of Browser from Scratch series, created to help me (and probably you too) to learn more about how a browser works by building one! - [**Read more**][1]

## Author

- [Viet Hung Nguyen](https://github.com/ZeroX-DG)

## License

- [MIT](LICENSE)

[1]: https://zerox-dg.github.io/blog/2020/05/29/Browser-from-Scratch-Introduction/
[2]: https://zerox-dg.github.io/blog/2020/09/01/Browser-from-Scratch-DOM-API/
