rustcass
========
I've included a bindgen  genereated cassandra.rs file, but you can rebuild it with "./build-ffi.sh"

The file "basic.rs" can be built with:
LD_LIBRARY_PATH=:/usr/lib/llvm-3.4/lib/:/usr/local/lib #if those are the correct paths for llvm and cassandra.so, respectively

 rustc src/cassandra.rs



Current state is very rough. Havnen't tried to clean it up or abstract away the C ugliness yet. The create keyspace, create table, and insert row commands all complete succesfully.
