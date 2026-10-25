# Result


```bash
cargo rustc --release --target x86_64-unknown-linux-gnu -- \
  -C link-arg=-nostartfiles \
  -C link-arg=-Wl,-T,asm/linker.ld \
  -C link-arg=-Wl,-e,_start

```

