# HDexecutor
```
$ cargo build --bin hdexecutor --release 
```

creates a binary that needs to be run with LD_LIBRARY_PATH=. and liblkl.so in .

first argument is serialized program to execute, second argument is the filesystem image path,
and the third is the filesystem type
