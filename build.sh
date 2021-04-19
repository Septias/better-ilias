#!/bin/bash
cd Backend
cargo build --release
echo "built backend"

cd ../Frontend
npm build
echo "built frontend"

cd ../

folder=build_$(date +%s)
mkdir builds/$folder

cp -r Frontend/dist builds/$folder
cp Backend/Rocket.toml Backend/target/release/better_ilias.exe builds/$folder