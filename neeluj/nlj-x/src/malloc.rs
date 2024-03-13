use std::{alloc::Layout, collections::BTreeMap};
use once_cell::sync::Lazy;

static mut MAP: Lazy<BTreeMap<*mut u8, Layout>> = Lazy::new(||BTreeMap::new());
#[export_name = "malloc"]
pub unsafe extern "C" fn malloc(a: usize) -> *mut u8 {
    let l = Layout::from_size_align(a, 1).unwrap();
    let a = std::alloc::alloc(l);
    MAP.insert(a,l);
    return a;
}
#[export_name = "free"]
pub unsafe extern "C" fn free(a: *mut u8){
    let l = *MAP.get(&a).unwrap();
    std::alloc::dealloc(a, l);
}
#[export_name = "len_of"]
pub unsafe extern "C" fn len_of(a: *mut u8) -> usize{
    return MAP.get(&a).unwrap().size();
}