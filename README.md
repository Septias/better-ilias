# BetterIlias
A tool for better interaction with the managment-website from Albert-Ludwigs-Universität Freiburg.

## Features
* interatact with ilias through a nice frontend which doesn't need 2sec to load
![image](https://user-images.githubusercontent.com/39526136/111999674-7c42d800-8b1d-11eb-8462-b31d891e3d5a.png)
* sync all files from ilias to local file system and open them through the frontend (no littered download folder)
* doesn't grill eyes when it's late

# Installation
If you don't want to build manually use this
* Windows: [better_ilias.zip](https://github.com/Septias/BetterIlias/files/6291894/better_ilias.zip)

otherwise:

### Backend
1. You need Rust to compile the better_ilias.exe, get it [here] (https://www.rust-lang.org/tools/install)
2. To compile the binary, run `cargo +nightly build --release` from within the `Backend`-folder. Your executable will be in `target/release/`

On windows, if you want don't want the console to shown you can go to `Backend/src/main.rs` and uncomment the second line (`//#![windows_subsystem = "windows"] -> #![windows_subsystem = "windows"]`). This will disable the creation of a window. The drawback is that there is no way to stop the execution other than manually killing it (e.g task manager).

### Frontend
1. To compile the Frontend you need [Node](https://nodejs.org/en/) which contains npm. 
2. Run `npm run build` from within the "Frontend"-Folder.
3. The compiled frontend will be in `Frontend/dist` 

### Putting it together
#### normal
1. Create a new folder "better_ilias"
2. Copy you executable in that folder.
3. Copy the `dist` folder from `Frondend` into "better_ilias"
4. Copy `Backend/Rocket.toml` into that folder.
5. Run it `./better_ilias.exe save_path="<some_path_maybe_onedrive>" 

#### dev
in dev mode just use `cargo run` in `Backend` and it will work. Literally same goes for the frontend. 

## Config
use `./better_ilias.exe save_path="<your-path>"` to chose a root dir for "Studium"-folder which contains ilias-content 
