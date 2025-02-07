#!/bin/bash

if [[ ! -d build ]]; then
    meson setup --cross-file=meson.cross build || exit $?
fi

meson compile -C build || exit $?
    
gdb_socket=${XDG_RUNTIME_DIR}/gdb.sock

nc -lkU ${gdb_socket} &

nc_pid=$!

qemu-system-riscv64 \
    -machine virt \
    -kernel build/kernel \
    -m 128M \
    -serial file:/dev/stdout \
    -nographic \
    -chardev socket,path=${gdb_socket},server=on,wait=off,id=gdb0 \
    -gdb chardev:gdb0 \
    "$@"

kill -9 ${nc_pid}
