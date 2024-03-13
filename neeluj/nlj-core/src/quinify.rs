use once_cell::sync::Lazy;
use waffle::ExportKind;

pub fn q() -> &'static [u8]{
    static mut V: Lazy<Vec<u8>> = Lazy::new(||vec![]);
    #[no_mangle]
    unsafe extern "C" fn q(x: u8){
        V.push(x)
    }
    return unsafe{
        &V
    };
}

pub fn quinify(m: &mut waffle::Module){
    let mut qs = q().to_vec();
    if qs.is_empty(){
        qs = std::fs::read("quinify_bake").unwrap();
    }
    for x in m.exports.clone(){
        if x.name == "q"{
            if let ExportKind::Func(f) = x.kind{
                quinify2::metaquin_iter(m, &qs, f);
            }
        }
    }
}