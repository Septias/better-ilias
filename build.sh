
cd Backend
cargo +nightly build --release
echo "built backend"

cd ../Frontend
npm run build
echo "built frontend"

cd ../

folder=builds/build_$(date +%s)
mkdir -p $folder

cp -r Frontend/dist $folder
cp Backend/Rocket.toml Backend/target/release/better_ilias.exe $folder

#just for me
cp -r $folder/* /d/dev/Installations/BetterIlias
