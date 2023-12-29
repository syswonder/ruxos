# INTRODUCTION
| App | Extra modules | Enabled features | Description |
|-|-|-|-|
| [udpserver](../apps/c/udpserver) | axalloc, axnet, axdriver, axtask | alloc, paging, net, multitask | UDP server test|

# RUN
``` bash
make A=apps/c/udpserver MUSL=y NET=y ARCH=aarch64 run 
```
# RESULT
``` 

       d8888                            .d88888b.   .d8888b.
      d88888                           d88P" "Y88b d88P  Y88b
     d88P888                           888     888 Y88b.
    d88P 888 888d888  .d8888b  .d88b.  888     888  "Y888b.
   d88P  888 888P"   d88P"    d8P  Y8b 888     888     "Y88b.
  d88P   888 888     888      88888888 888     888       "888
 d8888888888 888     Y88b.    Y8b.     Y88b. .d88P Y88b  d88P
d88P     888 888      "Y8888P  "Y8888   "Y88888P"   "Y8888P"

arch = aarch64
platform = aarch64-qemu-virt
target = aarch64-unknown-none-softfloat
smp = 1
build_mode = release 
log_level = warn

Hello, ArceOS C UDP server!
listen on: 0.0.0.0:5555
recv: 19 Bytes from 10.0.2.2:34354
Hello, UDP Server!
```
Then create a new terminal and run：
``` bash
chmod +x apps/c/udpserver/udpserver_test.sh
apps/c/udpserver/udpserver_test.sh
```
 This will send a message to port 5555 of localhost ， If the UDP service is running correctly the result will be：
 
 ```
Hello, ArceOS C UDP server!
listen on: 0.0.0.0:5555
recv: 19 Bytes from 10.0.2.2:40463
Hello, UDP Server!

recv: 1024 Bytes from 10.0.2.2:48431
Big message, it should fail#####################################################################################################################################################################################################################################################################################################################################################################################################################################################################################################################################################################################################################################################################################################################################################################################################################################################################################################################################################################################################################################
received message too long
 ```