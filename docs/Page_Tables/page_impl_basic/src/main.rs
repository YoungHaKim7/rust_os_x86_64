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
        address >= self.start_address() && address <= self.end_address()
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
