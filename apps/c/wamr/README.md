
# What's this?

This is an application to run wasm on Ruxos using [WAMR](https://github.com/bytecodealliance/wasm-micro-runtime), which is a wasm runtime developed by Intel and currently belongs to the [Bytecode Alliance](https://github.com/bytecodealliance).

The `main.wasm` and other wasm files is compiled from `.c` files in `rootfs/` using the WASM compiler. The `rootfs/` is a minimal rootfs for ruxos in this application.

# How to build?

The compilation of `WAMR` depends on `cmake`.

Take the 2048 game as an example. To run 2048, you need to compile the `2048.wasm` wasm file first.

We use `wasi-sdk` to compile the `2048` wasm file. You can download the `wasi-sdk` from [here](https://github.com/WebAssembly/wasi-sdk). Or you can use other wasm compiler.

In the path of 2048.c, use the following command to compile the `2048` wasm file:

```bash
<the path to `wasi-sdk`>/bin/clang -O3 -o 2048.wasm 2048.c
```

Or you can put the `2048.wasm` file from somewhere else into the rootfs.


# How to run?

After you have compiled the `.wasm` file, you can run it in ruxos.

- Run `HelloWorld`:

Run `HelloWorld` in ruxos using the following command:

```bash
make A=apps/c/wamr ARCH=aarch64 LOG=info SMP=4 run MUSL=y NET=y V9P=y V9P_PATH=apps/c/wamr/rootfs ARGS="iwasm,/main.wasm"
```

- Run the 2048 game:

Run the 2048 game in ruxos using the following command:

```bash
make A=apps/c/wamr ARCH=aarch64 LOG=info SMP=4 run MUSL=y NET=y V9P=y V9P_PATH=apps/c/wamr/rootfs ARGS="iwasm,/2048.wasm"
```

Input `A/B/C/D` to enjoy the game.

# Further

You can also run other wasm files in ruxos using this application. Just compile the `.wasm` file and put it into the `rootfs/` directory. Then run it using the command above, only change the `ARGS` parameter, and you can enjoy the wasm application in ruxos.
