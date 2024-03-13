use waffle::{ConstVal, GlobalData, InterpContext, MemoryData, MemorySegment, Module, TableData, Type};

pub fn render(i: &InterpContext, module: &mut Module) {
    for m in module.memories.iter().collect::<Vec<_>>() {
        let d = &i.memories[m];
        module.memories[m] = MemoryData {
            maximum_pages: Some(d.max_pages),
            initial_pages: (d.data.len() + 65535) / 65536,
            segments: vec![MemorySegment{offset: 0, data: d.data.clone()}]
        }
    }
    for g in module.globals.iter().collect::<Vec<_>>(){
        let d = &i.globals[g];
        module.globals[g].value = Some(match d{
            &ConstVal::I32(a) => a as u64,
            &ConstVal::F32(a) => a as u64,
            &ConstVal::I64(a) => a,
            &ConstVal::F64(a) => a,
            _ => unreachable!()
        })
    }
    for t in module.tables.iter().collect::<Vec<_>>(){
        let d = &i.tables[t];
        module.tables[t] = TableData{
            func_elements: Some(d.elements.clone()),
            max: None,
            ty: Type::FuncRef,
        }
    }
}
pub fn strip_start_interp(i: &mut InterpContext, module: &mut Module){
    if let Some(s) = module.start_func.take(){
        i.call(&module, s, &[]);
    }
}
pub fn interp<'a,'b,T>(m: &'a mut Module<'b>, go: impl FnOnce(&mut InterpContext,&mut Module<'b>) -> anyhow::Result<T>) -> anyhow::Result<T>{
    let mut ctx = InterpContext::new(m)?;
    let t = go(&mut ctx,m)?;
    render(&ctx,m);
    return Ok(t);
}