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
// pub mod syscall{
    use crate::{malloc::{free, len_of, malloc}};

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
// }