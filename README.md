# BetterIlias
A tool for better interaction with the managment-website from Albert-Ludwigs-Universität Freiburg.

## Features
* Interatact with ilias through a nice frontend which doesn't need 2sec to load
* Sync all files from ilias to local file system and open them through the frontend (no littered download folder)
* Doesn't grill eyes when it's late

### Installation
The easiest option is to download it from the [releases]()
The second option is to manually build it.

### Dev
#### Requirements
1. You need Rust to compile the better_ilias.exe, get it [here] (https://www.rust-lang.org/tools/install)
2. To compile the Frontend you need [Node](https://nodejs.org/en/) which contains npm. 

You can start the development process with 
```
RUST_LOG="info" cargo tauri dev
```
and build it with 
```
cargo tauri build
```

