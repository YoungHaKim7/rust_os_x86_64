# Result


```bash
Page:             Page { number: 4 }
Page number:      0x4
Start address:    0x4000
End address:      0x4fff
Contains 0x4234:  true
Contains 0x5000:  false
```


# basic

- In a Rust x86-64 operating system, you can represent a 4 KiB memory page with a `Page` type that stores its page number or starting virtual address.

Let's implement it from scratch, starting with a simple, runnable example and then showing how to use it in a kernel.

## 1. Implement a `Page` type in Rust

Create `src/main.rs`:

```rs

const PAGE_SIZE: u64 = 4096; // 4 KiB

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
struct Page {
    number: u64,
}

impl Page {
    /// Create a page from a page number.
    fn new(number: u64) -> Self {
        Self { number }
    }

    /// Create a page containing the given virtual address.
    fn containing_address(address: u64) -> Self {
        Self {
            number: address / PAGE_SIZE,
        }
    }

    /// Return the page's starting address.
    fn start_address(self) -> u64 {
        self.number * PAGE_SIZE
    }

    /// Return the page's ending address (inclusive).
    fn end_address(self) -> u64 {
        self.start_address() + PAGE_SIZE - 1
    }

    /// Return true if the address belongs to this page.
    fn contains(self, address: u64) -> bool {
        address >= self.start_address()
            && address <= self.end_address()
    }
}

fn main() {
    let page = Page::containing_address(0x4234);

    println!("Page:             {page:?}");
    println!("Page number:      {:#x}", page.number);
    println!("Start address:    {:#x}", page.start_address());
    println!("End address:      {:#x}", page.end_address());
    println!("Contains 0x4234:  {}", page.contains(0x4234));
    println!("Contains 0x5000:  {}", page.contains(0x5000));
}

```

Run it:

```bash
rustc main.rs -o page_demo
./page_demo
```

Expected output:

```bash
Page:             Page { number: 4 }
Page number:      0x4
Start address:    0x4000
End address:      0x4fff
Contains 0x4234:  true
Contains 0x5000:  false
```

## 2. How the implementation works

Suppose the virtual address is `0x4234`.

| Operation        | Calculation           | Result   |
| ---------------- | --------------------- | -------- |
| Find page number | `0x4234 / 0x1000`     | `4`      |
| Find page start  | `4 * 0x1000`          | `0x4000` |
| Find page end    | `0x4000 + 0x1000 - 1` | `0x4FFF` |
| Find offset      | `0x4234 - 0x4000`     | `0x234`  |

The address `0x4234` belongs to the page spanning `0x4000` through `0x4FFF`.

For 4 KiB pages, you can use bit operations instead of division and multiplication:

```
let page_number = address >> 12;
let page_start = address & !0xFFF;
let offset = address & 0xFFF;
```

These work because the page size is \\(2^{12}\\).

## 3. A more typical kernel implementation

In a real x86-64 kernel, it's useful to make the page size a compile-time constant and prevent accidental construction of unaligned page addresses.

```rs

const PAGE_SIZE: u64 = 4096;
const PAGE_MASK: u64 = PAGE_SIZE - 1;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
struct Page {
    start: u64,
}

impl Page {
    /// Construct a page from an aligned start address.
    fn from_start(start: u64) -> Option<Self> {
        if start & PAGE_MASK != 0 {
            return None;
        }

        Some(Self { start })
    }

    /// Find the page containing an address.
    fn containing(address: u64) -> Self {
        Self {
            start: address & !PAGE_MASK,
        }
    }

    fn start(self) -> u64 {
        self.start
    }

    fn end(self) -> u64 {
        self.start + PAGE_SIZE - 1
    }

    fn number(self) -> u64 {
        self.start / PAGE_SIZE
    }
}

fn main() {
    let page = Page::containing(0x1000_0042_34);

    println!("Page:  {page:?}");
    println!("Start: {:#x}", page.start());
    println!("End:   {:#x}", page.end());
    println!("Index: {:#x}", page.number());

    assert!(Page::from_start(0x4000).is_some());
    assert!(Page::from_start(0x4123).is_none());
}

```

Here, `from_start()` returns `None` if the supplied address is not aligned to a 4 KiB boundary. `containing()` automatically rounds an address down to its page boundary.

This version is still a conceptual model: production kernel code should also consider address-space type, canonical virtual addresses, overflow, and the possibility of larger pages.

## 4. How `Page` connects to page tables

A `Page` describes a virtual page; it does not allocate physical RAM or create a page-table mapping by itself.

| Type or component        | Responsibility                                                  |
| ------------------------ | --------------------------------------------------------------- |
| `Page`                   | Represents a page-sized region of virtual address space.        |
| `PhysFrame`              | Represents a page-sized region of physical memory.              |
| Page-table entry         | Maps a virtual page to a physical frame and stores permissions. |
| Page-table mapper        | Creates, removes, or queries mappings.                          |
| Physical frame allocator | Supplies available physical frames.                             |

For example, a kernel might eventually perform an operation conceptually like this:

```rs
// Conceptual API; these types and methods are not
// defined in the examples above.

let page = Page::containing(0x4000);
let frame = allocator.allocate_frame()?;

mapper.map(page, frame, writable = true)?;
```

The allocator finds a physical frame, and the mapper installs the corresponding page-table entry. Actual APIs vary by kernel implementation.

Remember: `Page` represents a region of virtual memory. To make that page access physical RAM, you need a physical frame and a valid page-table mapping.
