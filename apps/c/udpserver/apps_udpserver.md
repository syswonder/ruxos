# INTRODUCTION
| App | Extra modules | Enabled features | Description |
|-|-|-|-|
| [udpserver](../apps/c/udpserver) | axalloc, ruxnet, axdriver, axtask | alloc, paging, net, multitask | UDP server test|

# RUN
``` bash
make A=apps/c/udpserver MUSL=y NET=y ARCH=aarch64 run 
```
# RESULT
``` 
8888888b.                     .d88888b.   .d8888b.  
888   Y88b                   d88P" "Y88b d88P  Y88b 
888    888                   888     888 Y88b.      
888   d88P 888  888 888  888 888     888  "Y888b.   
8888888P"  888  888 `Y8bd8P' 888     888     "Y88b. 
888 T88b   888  888   X88K   888     888       "888 
888  T88b  Y88b 888 .d8""8b. Y88b. .d88P Y88b  d88P 
888   T88b  "Y88888 888  888  "Y88888P"   "Y8888P" 

arch = aarch64
platform = aarch64-qemu-virt
target = aarch64-unknown-none-softfloat
smp = 1
build_mode = release
log_level = warn

Hello, Ruxos C UDP server!
listen on: 0.0.0.0:5555
```
Then create a new terminal and run：
``` bash
chmod +x apps/c/udpserver/udpserver_test.sh
apps/c/udpserver/udpserver_test.sh
```
 This will send a message to port 5555 of localhost ， If the UDP service is running correctly the result will be：
 
 ```
Hello, Ruxos C UDP server!
listen on: 0.0.0.0:5555
recv: 1024 Bytes from 10.0.2.2:34774
Big message, it should fail#####################################################################################################################################################################################################################################################################################################################################################################################################################################################################################################################################################################################################################################################################################################################################################################################################################################################################################################################################################################################################################################
received message too long
 ```