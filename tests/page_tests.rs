use http_server::{PAGE_SIZE, PagePool};

#[test]
fn test_page_pool_acquisition_and_recycle() {
    let pool = PagePool::new(4);
    assert_eq!(pool.available(), 4);

    let mut page1 = pool.acquire();
    assert_eq!(page1.as_slice().len(), PAGE_SIZE);
    assert_eq!(pool.available(), 3);

    page1.as_mut_slice()[0] = 42;
    page1.as_mut_slice()[PAGE_SIZE - 1] = 99;
    assert_eq!(page1.as_slice()[0], 42);

    drop(page1);
    assert_eq!(pool.available(), 4);

    let page2 = pool.acquire();
    assert_eq!(pool.available(), 3);
    drop(page2);
    assert_eq!(pool.available(), 4);
}
