# BetterIlias
Modern Ui for Ilias with file downloads

## Features
* Responsive and modern frontend
* Synchronization of files to local filesystem for easier access
* Dark theme by default

## Installation
The easiest option is to download it from the [releases](https://github.com/Septias/better-ilias/releases)
The second option is to manually build it as described below.

## Dev
### Requirements
1. [Rust](https://www.rust-lang.org/tools/install)
2. [Node](https://nodejs.org/en/)

### Usage
You can start the development process with 
```
cargo tauri dev
```
or 
```
RUST_LOG="info" cargo tauri dev
```
to show logs.

Build the project using
```
cargo tauri build
```

