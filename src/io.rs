use std::alloc::{alloc, dealloc, Layout};
use std::os::unix::io::AsRawFd;
use std::io::Result;
use tokio_uring::fs::File;
use tokio_uring::buf::IoBuf;
use tokio_uring::buf::IoBufMut;

pub struct AlignedBuffer {
    ptr: *mut u8,
    layout: Layout,
    pub size: usize,
}

impl AlignedBuffer {
    pub const ALIGNMENT: usize = 4096;

    pub fn new(size: usize) -> Self {
        let adjusted_size = (size + Self::ALIGNMENT - 1) & !(Self::ALIGNMENT - 1);
        let layout = Layout::from_size_align(adjusted_size, Self::ALIGNMENT).expect("Invalid layout");
        let ptr = unsafe { alloc(layout) };
        if ptr.is_null() {
            panic!("Allocation failed");
        }
        AlignedBuffer { ptr, layout, size: adjusted_size }
    }
}

impl Drop for AlignedBuffer {
    fn drop(&mut self) {
        unsafe {
            dealloc(self.ptr, self.layout);
        }
    }
}

unsafe impl IoBuf for AlignedBuffer {
    fn stable_ptr(&self) -> *const u8 {
        self.ptr
    }

    fn bytes_init(&self) -> usize {
        self.size
    }

    fn bytes_total(&self) -> usize {
        self.size
    }
}

unsafe impl IoBufMut for AlignedBuffer {
    fn stable_mut_ptr(&mut self) -> *mut u8 {
        self.ptr
    }

    unsafe fn set_init(&mut self, _pos: usize) {
    }
}

pub struct DirectReader {
    file: File,
}

impl DirectReader {
    pub async fn open<P: AsRef<std::path::Path>>(path: P) -> Result<Self> {
        use std::os::unix::fs::OpenOptionsExt;
        let std_file = std::fs::OpenOptions::new()
            .read(true)
            .custom_flags(libc::O_DIRECT)
            .open(path)?;
        
        let file = File::from_std(std_file);
        Ok(DirectReader { file })
    }

    pub async fn read_at(&self, offset: u64, buffer: AlignedBuffer) -> (Result<usize>, AlignedBuffer) {
        self.file.read_at(buffer, offset).await
    }
}
