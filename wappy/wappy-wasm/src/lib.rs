

use anyhow::Context;
use waffle::Module;

#[derive(Clone,Debug,PartialEq,Eq)]
pub enum Value{
    WasmTy(waffle::Type),
    WasmVal(waffle::ConstVal),
    Tuple(Vec<Value>),
    Pick(Box<Value>, usize),
}

pub struct Augment{

}
impl Value{
    fn r#typeof(&self, m: &Module, a: &Augment) -> anyhow::Result<Value>{
        match self{
            Value::WasmTy(t) => anyhow::bail!("type where value is expected"),
            Value::WasmVal(v) => Ok(Value::WasmTy(match v{
                waffle::ConstVal::I32(_) => waffle::Type::I32,
                waffle::ConstVal::I64(_) => waffle::Type::I64,
                waffle::ConstVal::F32(_) => waffle::Type::F32,
                waffle::ConstVal::F64(_) => waffle::Type::F64,
                waffle::ConstVal::None => todo!(),
            })),
            Value::Tuple(t) => {
                let mut u = vec![];
                for t in t.iter(){
                    u.push(t.r#typeof(m, a)?);
                }
                return Ok(Value::Tuple(u));
            },
            Value::Pick(x, y) => {
                let Value::Tuple(x) = x.r#typeof(m, a)? else{
                    anyhow::bail!("not a tuple")
                };
                return Ok(x.get(*y).context("in getting tuple element")?.clone());

            },
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;


}
