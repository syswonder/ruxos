# Ruxos Architecture Overview

## Rustdoc

* https://github.com/syswonder/ruxos

## Ruxos Modules

* [rux9p](../modules/rux9p/): 9pfs integration for Ruxos.
* [axalloc](../modules/axalloc): Ruxos global memory allocator.
* [ruxconfig](../modules/ruxconfig): Platform-specific constants and parameters for Ruxos.
* [ruxdisplay](../modules/ruxdisplay): Ruxos graphics module.
* [ruxdriver](../modules/ruxdriver): Ruxos device drivers.
* [ruxfs](../modules/ruxfs): Ruxos filesystem module.
* [ruxhal](../modules/ruxhal): Ruxos hardware abstraction layer, provides unified APIs for platform-specific operations.
* [axlog](../modules/axlog): Macros for multi-level formatted logging used by Ruxos.
* [ruxnet](../modules/ruxnet): Ruxos network module.
* [ruxruntime](../modules/ruxruntime): Runtime library of Ruxos.
* [axsync](../modules/axsync): Ruxos synchronization primitives.
* [ruxtask](../modules/ruxtask): Ruxos task management module.

## Crates

* [allocator](../crates/allocator): Various allocator algorithms in a unified interface.
* [arm_gic](../crates/arm_gic): ARM Generic Interrupt Controller (GIC) register definitions and basic operations.
* [axerrno](../crates/axerrno): Error code definition used by Ruxos.
* [axfs_devfs](../crates/axfs_devfs): Device filesystem used by Ruxos.
* [axfs_vfs](../crates/axfs_vfs): Virtual filesystem interfaces used by Ruxos.
* [axio](../crates/axio): `std::io`-like I/O traits for `no_std` environment.
* [capability](../crates/capability): Provide basic capability-based security.
* [driver_9p](../crates/driver_9p/): Basic 9pfs operation traits for 9p protocol drivers.
* [driver_block](../crates/driver_block): Common traits and types for block storage drivers.
* [driver_common](../crates/driver_common): Device driver interfaces used by Ruxos.
* [driver_display](../crates/driver_display): Common traits and types for graphics device drivers.
* [driver_net](../crates/driver_net): Common traits and types for network device (NIC) drivers.
* [driver_pci](../crates/driver_pci): Structures and functions for PCI bus operations.
* [driver_virtio](../crates/driver_virtio): Wrappers of some devices in the `virtio-drivers` crate, that implement traits in the `driver_common` series crates.
* [flatten_objects](../crates/flatten_objects): A container that stores numbered objects. Each object can be assigned with a unique ID.
* [lazy_init](../crates/lazy_init): A wrapper for lazy initialized values without concurrency safety but more efficient.
* [linked_list](../crates/linked_list): Linked lists that supports arbitrary removal in constant time.
* [page_table](../crates/page_table): Generic page table structures for various hardware architectures.
* [page_table_entry](../crates/page_table_entry): Page table entry definition for various hardware architectures.
* [ratio](../crates/ratio): The type of ratios and related operations.
* [scheduler](../crates/scheduler): Various scheduler algorithms in a unified interface.
* [slab_allocator](../crates/slab_allocator): Slab allocator for `no_std` systems. Uses multiple slabs with blocks of different sizes and a linked list for blocks larger than 4096 bytes.
* [spinlock](../crates/spinlock): `no_std` spin lock implementation that can disable kernel local IRQs or preemption while locking.
* [timer_list](../crates/timer_list): A list of timed events that will be triggered sequentially when the timer expires.
* [tuple_for_each](../crates/tuple_for_each): Provides macros and methods to iterate over the fields of a tuple struct.

## Applications (Rust)

| App | Extra modules | Enabled features | Description |
|-|-|-|-|
| [helloworld](../apps/helloworld/) | | | A minimal app that just prints a string |
| [exception](../apps/exception/) | | paging | Exception handling test |
| [memtest](../apps/memtest/) | axalloc | alloc, paging | Dynamic memory allocation test |
| [display](../apps/display/) | axalloc, ruxdisplay | alloc, paging, display | Graphic/GUI test |
| [yield](../apps/task/yield/) | axalloc, ruxtask | alloc, paging, multitask, sched_fifo | Multi-threaded yielding test |
| [parallel](../apps/task/parallel/) | axalloc, ruxtask | alloc, paging, multitask, sched_fifo, irq | Parallel computing test (to test synchronization & mutex) |
| [sleep](../apps/task/sleep/) | axalloc, ruxtask | alloc, paging, multitask, sched_fifo, irq | Thread sleeping test |
| [priority](../apps/task/priority/) | axalloc, ruxtask | alloc, paging, multitask, sched_cfs | Thread priority test |
| [shell](../apps/fs/shell/) | axalloc, ruxdriver, ruxfs | alloc, paging, fs | A simple shell that responds to filesystem operations |
| [httpclient](../apps/net/httpclient/) | axalloc, ruxdriver, ruxnet | alloc, paging, net | A simple client that sends an HTTP request and then prints the response |
| [echoserver](../apps/net/echoserver/) | axalloc, ruxdriver, ruxnet, ruxtask | alloc, paging, net, multitask | A multi-threaded TCP server that reverses messages sent by the client  |
| [httpserver](../apps/net/httpserver/) | axalloc, ruxdriver, ruxnet, ruxtask | alloc, paging, net, multitask | A multi-threaded HTTP server that serves a static web page |
| [udpserver](../apps/net/udpserver/) | axalloc, ruxdriver, ruxnet | alloc, paging, net | A simple echo server using UDP protocol |

## Applications (C)
| App | Extra modules | Enabled features | Description |
|-|-|-|-|
| [helloworld](../apps/c/helloworld/) | | | A minimal C app that just prints a string |
| [memtest](../apps/c/memtest/) | axalloc | alloc, paging | Dynamic memory allocation test in C |
| [sqlite3](../apps/c/sqlite3/) | axalloc, ruxdriver, ruxfs | alloc, paging, fp_simd, fs | Porting of [SQLite3](https://sqlite.org/index.html) |
| [iperf](../apps/c/iperf/) | axalloc, ruxdriver, ruxfs, ruxnet | alloc, paging, fp_simd, fs, net, select | Porting of [iPerf3](https://iperf.fr/) |
| [redis](../apps/c/redis/) | axalloc, ruxdriver, ruxtask, ruxfs, ruxnet | alloc, paging, fp_simd, irq, multitask, fs, net, pipe, epoll | Porting of [Redis](https://redis.io/) |

## Dependencies

```mermaid
graph TD;
subgraph "User Apps"
A["Rust App"]
C["C App"]
end
subgraph "Ruxos ulib"
B["rust_libax"]
D["c_libax"]
E("rust_std")
F("c_musl")
end
A --> B;
C --> D;
A --> E;
D --> B;
E --> B;
E --> F;
C --> F;
subgraph "System compatible layer"
G["aeceos"]
H("linux")
end
B --> G;
F --> H;
subgraph "Ruxos modules"
I[ruxruntime]
J[axlog]
K[axsync]
L[ruxhal]
M[ruxconfig]
N[axalloc]
O[ruxtask]
P[ruxdriver]
Q[ruxnet]
Q1[ruxdisplay]
M1[ruxfs]
end
G --> I;
H --> I;
I --> IN6;
I --> IN3;
I --> IN9;
I --> IN4;
I --> IN15;
I --> N;
I --> M;
I --> P;
I --> L;
I --> J;
I --> Q;
I --> Q1;
I --> O;
Q1 --> P;
Q1 --> IN4;
Q1 --> K;
Q1 --> T5;
P --> IN11;
P --> T0;
P --> T1;
P --> T2;
P --> T3;
P --> T5;
P --> N;
P --> L;
P --> M;
N --> IN9;
N --> IN5;
N --> R;
N --> IN13;
L --> M;
L --> N;
L --> IN3;
L --> IN9;
L --> IN8;
L --> IN4;
L --> P1;
L --> P11;
L --> IN6;
L --> IN5;
L --> IN2;
L --> IN15;
J --> IN9;
J --> IN15;
Q --> T0;
Q --> T2;
Q --> IN4;
Q --> IN13;
Q --> L;
Q --> K;
Q --> O;
Q --> P;
K --> IN9;
K --> O;
O --> L;
O --> M;
O --> IN6;
O --> IN9;
O --> IN4;
O --> IN5;
O --> S;
O --> IN10;
O --> IN3;
O --> IN15;
M1 --> IN4;
M1 --> F3;
M1 --> T0;
M1 --> T1;
M1 --> IN14;
M1 --> IN13;
M1 --> F1;
M1 --> F2;
M1 --> P;
M1 --> K;
subgraph "Ruxos crates"
R[allocator]
IN12[arm_gic]
IN13[axerrno]
F1[axfs_devfs]
F2[axfs_vfs]
IN14[axio]
F3[capability]
IN15[crate_interface]
T1[driver_blk]
T0[driver_common]
T5[driver_display]
T2[driver_net]
T3[driver_virtio]
IN2[handler_table]
IN3[kernel_guard]
IN4[lazy_init]
Z[linked_list]
IN5[memory_addr]
P1[page_table]
P11[page_table_entry]
IN6[percpu]
IN7[percpu_macros]
IN8[ratio]
S[scheduler-FIFO_RR]
IN1[slab_allocator]
IN9[spinlock]
IN10[timer_list]
IN11[tuple_for_each]
T4(e1000)
V(smoltcp)
W(lwip_rust)
OUT2(bitmap-allocator)
Y(slab_allocator)
S1(FIFO)
S2(RR)
OUT1(buddy_system_allocator)
OUT3(virtio-drivers)
end
R --> OUT1;
R --> OUT2;
R --> Y;
IN4 --> IN13;
S --> Z;
OUT1 --> Z;
OUT2 --> Z;
IN1 --> OUT1;
Y --> Z;
T3 --> T1;
T3 --> T2;
T3 --> T5;
T3 --> OUT3;
T3 --> T0;
T1 --> T0;
T2 --> T0;
T4 --> T0;
T5 --> T0;
IN3 --> IN15;
P1 --> IN5;
P1 --> P11;
P11 --> IN5;
IN6 --> IN3;
IN6 --> IN7;
IN9 --> IN3;
F3 --> IN13;
F2 --> IN13;
F1 --> F2;
```
