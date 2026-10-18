# Result

- https://github.com/rust-osdev/bootloader/blob/main/examples/basic/basic-os.md

```bash
cargo b

$ cargo run bios

$ cargo run uefi

just qemu

```


## 내가 만든 이미지 파일 찾기

```bash
find target -type f \
    \( -name 'a02_hello_world' -o -name '*.elf' -o -name '*.iso' -o -name '*.img' \) \
    -print
```
