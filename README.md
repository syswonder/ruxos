# RuxOS

<p align="center">
    <img src="doc/figures/ruxos-logo0.svg" alt="RuxOS-logo" width="500"><br>
    A unikernel operating system written in Rust.<br/>
    <a href="https://github.com/syswonder/ruxos/actions/workflows/build.yml"><img src="https://github.com/syswonder/ruxos/actions/workflows/build.yml/badge.svg?branch=main" alt="Build CI" style="max-width: 100%;"></a>
    <a href="https://github.com/syswonder/ruxos/actions/workflows/test.yml"><img src="https://github.com/syswonder/ruxos/actions/workflows/test.yml/badge.svg?branch=main" alt="Test CI" style="max-width: 100%;"></a>
    <br/>
</p>

RuxOS was inspired by [Unikraft](https://github.com/unikraft/unikraft) and [ArceOS](https://github.com/rcore-os/arceos)

🚧 Working In Progress. See [RuxOS Book](https://ruxos.syswonder.org) for more information.

## Features & TODOs

* [x] Architecture: x86_64, riscv64, aarch64
* [x] Platform: QEMU pc-q35 (x86_64), virt (riscv64/aarch64)
* [x] Multi-thread
* [x] FIFO/RR/CFS scheduler
* [x] VirtIO net/blk/gpu drivers
* [x] TCP/UDP net stack using [smoltcp](https://github.com/smoltcp-rs/smoltcp)
* [x] Synchronization/Mutex
* [x] SMP scheduling with single run queue
* [x] File system
* [x] Compatible with Linux apps
* [x] Dynamically loading apps
* [ ] Interrupt driven device I/O
* [ ] Async I/O

## Example apps

Example applications can be found in the [apps/](apps/) directory. All applications must at least depend on the following modules, while other modules are optional:

* [ruxruntime](modules/ruxruntime/): Bootstrapping from the bare-metal environment, and initialization.
* [ruxhal](modules/ruxhal/): Hardware abstraction layer, provides unified APIs for cross-platform.
* [ruxconfig](modules/ruxconfig/): Platform constants and kernel parameters, such as physical memory base, kernel load addresses, stack size, etc.
* [axlog](modules/axlog/): Multi-level formatted logging.

The currently supported applications and programming languages, as well as their dependent modules and features, are shown in the following table:

### Rust applications

| App | Extra modules | Enabled features | Description |
|-|-|-|-|
| [display](apps/display/) | axalloc, ruxdisplay | alloc, paging, display | Graphic/GUI test |
| [shell](apps/fs/shell/) | axalloc, ruxdriver, ruxfs | alloc, paging, fs | A simple shell that responds to filesystem operations |

### C applications

| App | Enabled features | Description |
|-|-|-|
| [helloworld](apps/c/helloworld/) | | A minimal app that just prints a string by C |
| [envtest](apps/c/envtest/) | alloc, paging | An environment variable test |
| [memtest](apps/c/memtest/) | alloc, paging | Dynamic memory allocation test by C |
| [filetest](apps/c/filetest/) | alloc, paging, fs, blkfs | File system operation test |
| [httpclient](apps/c/httpclient/) | alloc, paging, net | A simple client that sends an HTTP request and then prints the response by C |
| [httpserver](apps/c/httpserver/) | alloc, paging, net | A multi-threaded HTTP server that serves a static web page by C |
| [udpserver](apps/c/udpserver/) | alloc, paging, net | A simple UDP server that send back original message |
| [systime](apps/c/systime/) | rtc | A simple test for real time clock module |
| [basic](apps/c/pthread/basic/) | alloc, paging, multitask, irq | A simple test for basic pthread-related API in C standard library |
| [parallel](apps/c/pthread/parallel/) | alloc, paging, multitask | Parallel computing test to test synchronization by C |
| [pipe](apps/c/pthread/pipe/) | alloc, paging, multitask, pipe | A test for pipe API |
| [sleep](apps/c/pthread/sleep/) | alloc, paging, multitask, irq | Thread sleeping test |
| [tsd](apps/c/pthread/tsd/) | alloc, paging, multitask, irq | A test for pthread-key related API |
| [dl](apps/c/dl/) | paging, alloc, irq, musl, multitask, fs, pipe, poll, rtc, signal, virtio-9p | An example for dynamically loading apps |
| [libc-bench](apps/c/libc-bench/) | alloc, multitask, fs, musl | A standard libc test for musl libc integration |
| [sqlite3](apps/c/sqlite3/) | alloc, paging, fs, fp_simd, blkfs | A simple test for Sqlite3 API |
| [iperf](https://github.com/syswonder/rux-iperf) | alloc, paging, net, fs, blkfs, select, fp_simd | A network performance test tool |
| [redis](https://github.com/syswonder/rux-redis) | alloc, paging, fp_simd, irq, multitask, fs, blkfs, net, pipe, epoll, poll, virtio-9p, rtc | Redis server on Ruxos |
| [cpp](apps/c/cpp/) | alloc, paging, irq, multitask, fs, random-hw | C++ benchmark |
| [nginx](https://github.com/syswonder/rux-nginx) | alloc, paging, fp_simd, irq, multitask, fs, blkfs, net, pipe, epoll, poll, select, rtc, signal | Run Nginx as web server |
| [wamr](https://github.com/syswonder/rux-wamr) | alloc, paging, fp_simd, irq, multitask, fs, virtio-9p, signal, smp | Wasm runtime |

### Programming languages

| Language | Description |
|- | - |
| C | Run C apps by RuxOS ruxlibc or standard musl libc supported by ruxmusl. Evaluated by libc-bench. |
| C++ | Run C++ apps by c++ static library provided by musl libc. Passed c++ benchmark. Evaluated by c++ benchmark. |
| [Perl](https://github.com/syswonder/rux-perl) | Run Perl standard library by musl libc. Evaluated by Perl benchmark. |
| [Python](https://github.com/syswonder/rux-python3) | Run Python apps by dynamically loading Python modules. Evaluated by Python benchmark. |
| Rust | Run Rust standard library by modifying Rust std source. Evaluated by Rust tests. |

## Build & Run

### Install build dependencies

Install [cargo-binutils](https://github.com/rust-embedded/cargo-binutils) to use `rust-objcopy` and `rust-objdump` tools:

```bash
cargo install cargo-binutils
```

#### for build&run C apps
Install `libclang-dev`:

```bash
sudo apt install libclang-dev
```

Download&Install `cross-musl-based toolchains`:
```
# download
wget https://musl.cc/aarch64-linux-musl-cross.tgz
wget https://musl.cc/riscv64-linux-musl-cross.tgz
wget https://musl.cc/x86_64-linux-musl-cross.tgz
# install
tar zxf aarch64-linux-musl-cross.tgz
tar zxf riscv64-linux-musl-cross.tgz
tar zxf x86_64-linux-musl-cross.tgz
# exec below command in bash OR add below info in ~/.bashrc
export PATH=`pwd`/x86_64-linux-musl-cross/bin:`pwd`/aarch64-linux-musl-cross/bin:`pwd`/riscv64-linux-musl-cross/bin:$PATH
```

### Example apps

```bash
# in ruxos directory
make A=path/to/app ARCH=<arch> LOG=<log>
```

Where `<arch>` should be one of `riscv64`, `aarch64`，`x86_64`.

`<log>` should be one of `off`, `error`, `warn`, `info`, `debug`, `trace`.

`path/to/app` is the relative path to the example application.

More arguments and targets can be found in [Makefile](Makefile).

For example, to run the [shell](apps/fs/shell/) on `qemu-system-aarch64`:

```bash
make A=apps/fs/shell ARCH=aarch64 LOG=info run BLK=y
```

Note that the `BLK=y` argument is required to enable the block device in QEMU. These arguments (`NET`, `GRAPHIC`, etc.) only take effect at runtime not build time.

### Your custom apps

#### Rust

1. Create a new rust package with `no_std` and `no_main` environment.
2. Add `axstd` dependency and features to enable to `Cargo.toml`:

    ```toml
    [dependencies]
    axstd = { path = "/path/to/ruxos/ulib/axstd", features = ["..."] }
    ```

3. Call library functions from `axstd` in your code, just like the Rust [std](https://doc.rust-lang.org/std/) library.
4. Build your application with RuxOS, by running the `make` command in the application directory:

    ```bash
    # in app directory
    make -C /path/to/ruxos A=$(pwd) ARCH=<arch> run
    # more args: LOG=<log> SMP=<smp> NET=[y|n] ...
    ```

    All arguments and targets are the same as above.

#### C

1. Create `axbuild.mk` and `features.txt` in your project:

    ```bash
    app/
    ├── foo.c
    ├── bar.c
    ├── axbuild.mk      # optional, if there is only one `main.c`
    └── features.txt    # optional, if only use default features
    ```

2. Add build targets to `axbuild.mk`, add features to enable to `features.txt` (see this [example](apps/c/sqlite3/)):

    ```bash
    # in axbuild.mk
    app-objs := foo.o bar.o
    ```

    ```bash
    # in features.txt
    alloc
    paging
    net
    ```

3. Build your application with RuxOS, by running the `make` command in the application directory:

    ```bash
    # in app directory
    make -C /path/to/ruxos A=$(pwd) ARCH=<arch> run
    # more args: LOG=<log> SMP=<smp> NET=[y|n] ...
    ```

### How to build RuxOS for specific platforms and devices

Set the `PLATFORM` variable when run `make`:

```bash
# Build helloworld for raspi4
make PLATFORM=aarch64-raspi4 A=apps/helloworld
```

You may also need to select the corrsponding device drivers by setting the `FEATURES` variable:

```bash
# Build the shell app for raspi4, and use the SD card driver
make PLATFORM=aarch64-raspi4 A=apps/fs/shell FEATURES=driver-bcm2835-sdhci
# Build Redis for the bare-metal x86_64 platform, and use the ixgbe and ramdisk driver
make PLATFORM=x86_64-pc-oslab A=apps/c/redis FEATURES=driver-ixgbe,driver-ramdisk SMP=4
```

## RuxGo

A convient tool to run RuxOS applications by concise command. See [RuxGo-Book](https://ruxgo.syswonder.org/) for more information.

## Design

![](doc/figures/ruxos.svg)
