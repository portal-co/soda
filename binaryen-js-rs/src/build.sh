cd $(dirname $0)
git clone --branch version_116 --recurse-submodules https://github.com/WebAssembly/binaryen.git $OUT_DIR/binaryen
old=$(pwd)
cd "$HOME/emsdk"
. $HOME/emsdk/emsdk_env.sh
cd "$old"
emcc --version
mkdir $OUT_DIR/binaryen/build
emcmake cmake -B $OUT_DIR/binaryen/build -S $OUT_DIR/binaryen -DCMAKE_BUILD_TYPE=Release -DCMAKE_EXE_LINKER_FLAGS="-sMAXIMUM_MEMORY=4294967296"
emmake make -C $OUT_DIR/binaryen/build binaryen_js+
