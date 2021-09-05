cargo build --release --target=armv7a-none-eabi
cp ./target/armv7a-none-eabi/release/librustapp.a ../
ar -d ../librustapp.a compiler_builtins-e989fd3c24f0ae44.compiler_builtins.ak41kaiv-cgu.60.rcgu.o
