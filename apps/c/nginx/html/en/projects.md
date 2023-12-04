# Syswonder Open Source Projects


## rcore

Reusable operating system kernel modules implemented in Rust.

Mail list: [bulletin@syswonder.org](https://maillist.syswonder.org/mailman3/lists/bulletin.syswonder.org/)

## sysHyper

SysHyper is a separation kernel hypervisor implemented in Rust language.
It is highly simplified and optimized for time and space partitioning.
It is loaded by a Linux system. Once activated, it runs bare-metal, and
splits off parts of the system' resources and assigns them to Unikernel
OSs in different zones. The SysHyper design references much from [jailhouse](https://github.com/siemens/jailhouse).

Mail list: [hypervisor@syswonder.org](https://maillist.syswonder.org/mailman3/lists/hypervisor.syswonder.org/)

## rukos

rukos (Rust UniKernel OS) is a [Unikernel](https://en.wikipedia.org/wiki/Unikernel) operating system, supporting Linux applications. rukos is built from the kernel framework [ArceOS](https://github.com/rcore-os/arceos). ArceOS defines a set of interfaces among different os modules. rukos addes/optimizes/replaces necessary modules to meet the requirements of different ubiquitous applications. As ArceOS, rukos is developped in type-safe Rust language. 

Repo: [rukos@github](https://github.com/syswonder/rukos) (open soon)

Mail list: [unikernel@syswonder.org](https://maillist.syswonder.org/mailman3/lists/unikernel.syswonder.org/)

