use std::sync::{Arc, Mutex};

pub const PAGE_SIZE: usize = 4096;

pub struct Page {
    buf: Option<Box<[u8; PAGE_SIZE]>>,
    pool: Arc<Mutex<Vec<Box<[u8; PAGE_SIZE]>>>>,
}

impl Page {
    /// # Panics
    /// Panics if page buffer was taken
    #[must_use]
    pub fn as_slice(&self) -> &[u8] {
        self.buf.as_deref().expect("page buffer present")
    }

    /// # Panics
    /// Panics if page buffer was taken
    pub fn as_mut_slice(&mut self) -> &mut [u8] {
        self.buf.as_deref_mut().expect("page buffer present")
    }
}

impl Drop for Page {
    fn drop(&mut self) {
        if let Some(mut block) = self.buf.take() {
            block.fill(0);
            if let Ok(mut list) = self.pool.lock() {
                list.push(block);
            }
        }
    }
}

#[derive(Clone)]
pub struct PagePool {
    pages: Arc<Mutex<Vec<Box<[u8; PAGE_SIZE]>>>>,
}

impl PagePool {
    #[must_use]
    pub fn new(capacity: usize) -> Self {
        let mut list = Vec::with_capacity(capacity);
        for _ in 0..capacity {
            list.push(Box::new([0_u8; PAGE_SIZE]));
        }
        Self {
            pages: Arc::new(Mutex::new(list)),
        }
    }

    #[must_use]
    pub fn acquire(&self) -> Page {
        let block = self
            .pages
            .lock()
            .ok()
            .and_then(|mut list| list.pop())
            .unwrap_or_else(|| Box::new([0_u8; PAGE_SIZE]));

        Page {
            buf: Some(block),
            pool: Arc::clone(&self.pages),
        }
    }

    #[must_use]
    pub fn available(&self) -> usize {
        self.pages.lock().map_or(0, |list| list.len())
    }
}
