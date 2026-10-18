# 이미지 만든거 파일 찾기

```bash
find target -type f \
    \( -name 'a02_hello_world' -o -name '*.elf' -o -name '*.iso' -o -name '*.img' \) \
    -print
```


- 기타 파일 찾기

```bash
find . -maxdepth 3 -type f \
    \( -name 'config.toml' -o -name 'justfile' -o -name 'Makefile' \
       -o -name 'linker.ld' -o -name 'boot.S' -o -name 'main.rs' \)
```


# 최종 build
- https://github.com/rust-osdev/bootloader/blob/main/examples/basic/basic-os.md

```
cargo b

$ cargo run bios

$ cargo run uefi

just qemu
```

- `just qemu`

```justfile
project_name := `basename "$(pwd)"`
# target_machine := "x86_64-unknown-none"

qemu:
    qemu-system-x86_64 \
        -m 512M \
        -drive format=raw,file=target/debug/build/a02_hello_world/7fabad4d744cc494/out/bios.img \
        -boot order=c \
        -vga std
```
