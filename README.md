# ! Broken
Better ilias is not working atm because it changed it's login-form. I need to redo the login-flow before updates from Ilias can be loaded

#### 01.11.2021
There is a meeting planned with the ILIAS-staff planned of 28.11.2021 to maybe find a solution on why the login-flow is not working anylonger and how to fix it.

# BetterIlias
A tool for better interaction with the managment-website from Albert-Ludwigs-Universität Freiburg.


## Features
* interatact with ilias through a nice frontend which doesn't need 2sec to load
![image](https://user-images.githubusercontent.com/39526136/111999674-7c42d800-8b1d-11eb-8462-b31d891e3d5a.png)
* sync all files from ilias to local file system and open them through the frontend (no littered download folder)
* doesn't grill eyes when it's late
* create notes for your different courses  
![image](https://user-images.githubusercontent.com/39526136/114848177-b894eb80-9dde-11eb-9022-939e089322da.png)

# Installation
If you don't want to build manually use this
* Windows: [build_1619173743.zip](https://github.com/Septias/BetterIlias/files/6364627/build_1619173743.zip)
* MacOs: [build_1619191947_macos.zip](https://github.com/Septias/BetterIlias/files/6366910/build_1619191947_macos.zip)

otherwise:

### Requirement
1. You need Rust to compile the better_ilias.exe, get it [here] (https://www.rust-lang.org/tools/install)
2. To compile the Frontend you need [Node](https://nodejs.org/en/) which contains npm. 

### Putting it together
If you have bash installed you can use `build.sh` to create the ilias-folder.

Otherwise:
1. Create a new folder "better_ilias"
2. Copy you executable in that folder.
3. Copy the `dist` folder from `Frontend` into "better_ilias"
4. Copy `Backend/Rocket.toml` into that folder.
5. Run it `./better_ilias.exe save_path="<some_path_maybe_onedrive>"` from **within** the new folder

#### dev
in dev mode just use `cargo run` in `Backend` and it will work. Literally same goes for the frontend. 

## Config
use `./better_ilias.exe save_path="<your-path>"` to chose a root dir for "Studium"-folder which contains ilias-content 

On windows, if you want don't want the console to shown you can go to `Backend/src/main.rs` and uncomment the second line (`//#![windows_subsystem = "windows"] -> #![windows_subsystem = "windows"]`). This will disable the creation of a window. The drawback is that there is no way to stop the execution other than manually killing it (e.g task manager).

