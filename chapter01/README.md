# 완성된 코드 x86_64 OS(Rust로 만든거)

- https://github.com/phil-opp/blog_os/tree/post-12

# Writing an OS in Rust
- https://os.phil-opp.com/

# 누가 만들고 있는거 os.phil보다 더 업그레이드 버젼
- https://github.com/RustOS-Dev/RustOS

# basic-os
- https://github.com/rust-osdev/bootloader/blob/main/examples/basic/basic-os.md

- If you're building an x86-64 virtual machine in Rust and plan to run it with `qemu-system-x86_64`, I recommend starting with a small, well-established bare-metal operating-system environment rather than a full embedded Linux distribution.

Since you're working with Rust bootloader and ARM architecture concepts too, a good learning path is to use a bootable x86-64 image, implement the low-level components in Rust, and run everything in QEMU.

## 1. Recommended environments

[Rust Icon and Logo Set for Modern Design](https://images.openai.com/static-rsc-4/_Ud88glhl17y28zMSDFYIOgcfjQulE_RW-xWSAa0LmyADbYTnyC65PVukdz9fNyBb3Pz5eXcLrn4M1OIzDPCRrQQvTJqrpwUAP5UWVTfg-dZ3OdJiiH_PPVlGd3fKmWShnao6Z_tn-JfhI160HENPZM7brXSXSF-iNk3akisHJE?purpose=inline)

1\. Writing an OS in Rust — best for learning

My recommendation

A widely used tutorial for building a small `no_std` x86-64 kernel in Rust. It covers booting, VGA output, interrupts, memory management, and QEMU testing.

Best if you want to build your own kernel from scratch.

[Official tutorial](https://os.phil-opp.com/?utm_source=chatgpt.com) · [Rust bootloader](https://github.com/rust-osdev/bootloader?utm_source=chatgpt.com)

[QEMU monitoring & observability | Dynatrace Hub](https://images.openai.com/static-rsc-4/HbQyKCkyidJ5Lsq1M67yorwt4ygw5dkfp3zjJZznoGoGRrK7i4WziV_Ewbjs9KqBpFx6JCDROEnOBQdywYClMegZTCTXObmJOaXvu8_begJhOGIK1a3ZE1PsuI7GU2bcrk9u76cQ4I67S1OBdFHDrN33eKjXtIJaB4_JAVIjtuw?purpose=inline)

2\. QEMU + a custom Rust kernel — best for bare metal

Use `qemu-system-x86_64` to emulate a PC with virtual CPUs, RAM, firmware, disks, and serial output. Your Rust kernel can run without a host operating system inside the guest.

[QEMU x86 system emulation documentation](https://www.qemu.org/docs/master/system/target-i386.html?utm_source=chatgpt.com)

[Alpine Linux: Great Even Outside of Containers | by Umair | Medium](https://images.openai.com/static-rsc-4/RSgCyEqrmTO4d5_RnxfZLFCRax4_U3CSRLryjEY3Yg55oX0LzEqlbY-YetVCPm9uTB17Lv-FclQpJyu-mVpcYtGAtuQUhfkChgmSrBooIVXyjCqhVUIM4KqbOpR29cCz1l8ilKjnND18PBOw8aRWLhH8XFktuMgs5MoXhvGNLk8?purpose=inline)

3\. Alpine Linux — best for a lightweight guest OS

A small Linux distribution suitable for experiments involving a guest OS, Rust applications, networking, and system services.

Best if you want a usable Linux VM rather than implementing the kernel yourself.

[Official website](https://alpinelinux.org/?utm_source=chatgpt.com)

[What is Buildroot? | VOCAL Technologies](https://images.openai.com/static-rsc-4/kxIeVj2Rx9twq_jXnuf4NJ_gyinx8zebPFTEhv5dcm3hHgrz1EqHGsxcBGUOlXRhivgFkAfs_0WU6PVIWZVxWt6B9wSvzEeBCN4nTvweseJl0rtMcoXyBIP8wqbZZwy2-gi-QnlMRfZNO8qyM9MB1cQqy_LSDnueqhQMc_tqNKI?purpose=inline)

4\. Buildroot — best for embedded Linux

Build a minimal Linux system with a kernel, root filesystem, and selected utilities. This is useful for learning embedded boot flows and constructing reproducible guest images.

[Official website](https://buildroot.org/?utm_source=chatgpt.com)

## 2. My recommended setup

For your use case, I'd start with the Rust `bootloader` project and target `x86_64-unknown-none`. The bootloader project provides a minimal BIOS/UEFI boot path for Rust kernels and includes a runnable QEMU example.&#x20;

[image](https://www.google.com/s2/favicons?domain=https://github.com\&sz=32)

GitHub

+1



```
Host machine
└── Rust toolchain
    ├── cargo
    ├── rustc (nightly)
    ├── x86_64-unknown-none
    └── bootloader
          │
          ▼
    Bootable disk image
          │
          ▼
    qemu-system-x86_64
          │
          ▼
    Emulated x86-64 PC
    ├── Virtual CPU
    ├── RAM
    ├── Firmware (BIOS/UEFI)
    ├── Serial port
    └── Rust kernel (no_std)
```

### Install the basic tools

On Linux (Debian/Ubuntu):

```
sudo apt update
sudo apt install qemu-system-x86 ovmf

rustup toolchain install nightly
rustup component add llvm-tools-preview --toolchain nightly
rustup target add x86_64-unknown-none --toolchain nightly
```

Then follow the bootloader's basic example:

```
git clone https://github.com/rust-osdev/bootloader.git
cd bootloader
cargo run --example basic -- bios
```

The exact example command can vary with the repository version; its [basic OS example](https://github.com/rust-osdev/bootloader/blob/main/examples/basic/basic-os.md?utm_source=chatgpt.com) documents the build and launch procedure.&#x20;

[image](https://www.google.com/s2/favicons?domain=https://github.com\&sz=32)

GitHub



### Run QEMU directly

Once you have a bootable disk image, you can launch the VM with a command such as:

```
qemu-system-x86_64 \
    -machine q35 \
    -m 512M \
    -smp 2 \
    -drive format=raw,file=os.img \
    -serial stdio \
    -no-reboot
```

Here, `os.img` is a placeholder for your actual bootable image. This command assumes the image is compatible with the selected machine and firmware configuration. Add an appropriate BIOS or UEFI configuration if needed.

QEMU supports different machine models and CPU emulation modes, so you can later expand the VM with virtual disks, network devices, and debugging options.&#x20;

[image](https://www.google.com/s2/favicons?domain=https://www.qemu.org\&sz=32)

QEMU documentation

+1



## 3. Which environment should you choose?

| Your goal                                                | Recommended choice                                                    |
| -------------------------------------------------------- | --------------------------------------------------------------------- |
| Learn x86-64 architecture and write a kernel in Rust     | Rust bootloader + custom `no_std` kernel                              |
| Learn embedded Linux boot and root filesystems           | Buildroot                                                             |
| Run a lightweight, practical Linux guest                 | Alpine Linux                                                          |
| Study OS internals with a more complete existing Rust OS | [RustOS](https://github.com/RustOS-Dev/RustOS?utm_source=chatgpt.com) |

One important question: When you say “embedded environment,” do you mean an embedded Linux system, or do you want to write your own operating-system kernel in Rust and boot it inside QEMU?

That distinction determines whether you should start with Buildroot or the Rust bootloader approach.
