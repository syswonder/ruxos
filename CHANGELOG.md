# CHANGELOG

## [v0.1.0] - 2025-07

### ✨ New Features
#### Architecture Support
- **RISC-V64** (#196)
  - mmap & process fork implementation
  - Toolchain compatibility fixes
- **ARM Enhancements** (#183)
  - GICv3 interrupt controller
  - Device Tree Blob (DTB) parsing

#### Core Systems
- **Filesystem & Storage**
  - FUSE integration (#201)
  - New syscalls: `fchmodat` (#190), `ftruncate` (#187)
  - VFS refactoring with ext4/fatfs support (#152)
  - Unified file/directory implementations (#186)
  - FIFO/named pipes (#185) & PTY support (#189)
- **Networking & IPC**
  - UNIX socket enhancements (#195)
  - DGRAM support (#164) & `socketpair` (#171)
  - Static routing with loopback support (#179)
- **Device Drivers**
  - Virtio subsystem updates (#205)
  - Virtio-console support (#143)
  - TTY/Termios improvements (#181)
- **Process Management**
  - Signal handling (#178) & `rt_sigaction` (#131)
  - CLOEXEC flag support (#162)

### 🐛 Bug Fixes
- Fixed `rename` operations (#202)
- Resolved busybox page faults (#194)
- Fixed network buffer memory leaks (#173)
- Corrected page fault handling during nested forks (#151)
- Fixed `wait4` null pointer dereference (#166)
- Resolved file close scheduling deadlock (#175)
- Corrected `getsockopt` implementation (#170)
- Fixed percpu crate alignment (#184)

### ⚙️ Infrastructure
- Toolchain upgraded to `nightly-2025-05-07` (#197)

### 📦 Applications
- Support for sshd after PR #202

---

## [v0.0.3] - Initial Release

### 🚀 Core Capabilities
- **Architectures**: x86_64 • AArch64 • RiscV64
- **Platforms**: QEMU pc-q35 (x86) • virt (RiscV/ARM)
- **Schedulers**: FIFO • RR • CFS
- **Drivers**: VirtIO (net, blk, gpu, 9p)
- **Networking**: TCP/UDP stack (smoltcp/LwIP)
- **Filesystems**: fatfs • ramfs • 9pfs • devfs
- **Dynamic App Loading** (#97)

### 📦 Supported Applications
| Application | Functionality | Repository |
|-------------|---------------|------------|
| **Redis** | Server with `redis-cli` & benchmark | [syswonder/rux-redis](https://github.com/syswonder/rux-redis) |
| **Nginx** | HTTP/HTTPS web server | [syswonder/rux-nginx](https://github.com/syswonder/rux-nginx) |
| **Wamr** | WASM execution + wasi-nn | [syswonder/rux-wamr](https://github.com/syswonder/rux-wamr) |
| **Iperf** | Network performance testing | [syswonder/rux-iperf](https://github.com/syswonder/rux-iperf) |
| **Sqlite** | Embedded SQL database | |
| **Python** | Python 3 runtime | [syswonder/rux-python3](https://github.com/syswonder/rux-python3) |

### 🛠️ Development Ecosystem
- **Languages**: Rust • C/C++ (musl/ruxlibc) • Perl
- **Tools**: [RuxGo](https://github.com/syswonder/ruxgo)
- **Documentation**:
  - [RuxOS-Book](https://ruxos.syswonder.org)
  - [RuxGo-Book](https://ruxgo.syswonder.org)

> **Get Started**: We recommend beginning with the [RuxOS-Book](https://ruxos.syswonder.org)
