pub mod malloc {
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
}
pub mod syscall_abi{
    #[link(wasm_import_module = "i")]
    extern "C"{
        pub fn syscall(a: *mut u8) -> *mut u8;
    }
    #[export_name = "__syscall_wrapper"]
    unsafe extern "C" fn sys_wrapper(a: *mut u8) -> *mut u8{
        return syscall(a);
    }
}
pub mod syscall{
    use crate::{malloc::{free, len_of, malloc}, syscall_abi};

    pub fn syscall(a: &[u8]) -> Vec<u8>{
        unsafe{
            let mut m = malloc(a.len());
            std::slice::from_raw_parts_mut(m, a.len()).copy_from_slice(a);
            let s = syscall_abi::syscall(m);
            let mut v = vec![0; len_of(s)];
            v.copy_from_slice(std::slice::from_raw_parts(s, len_of(s)));
            free(s);
            return v;
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
}
