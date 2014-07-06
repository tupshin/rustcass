LD_PRELOAD=/usr/lib/llvm-3.4/lib/libclang.so ../rust-bindgen/bindgen -l cassandra -match cassandra.h -o src/cassandra.rs /usr/local/include/cassandra.h 
