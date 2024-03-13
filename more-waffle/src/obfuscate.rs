use waffle::{
    Block, FunctionBody, Global, GlobalData, Memory, MemoryArg, MemoryData, Module, Operator, Type,
    Value,
};

pub mod cff;

pub struct Stack {
    pub mem: Memory,
    pub glob: Global,
}
impl Stack {
    pub fn new(module: &mut Module, size: usize) -> Self {
        let g = module.globals.push(GlobalData {
            ty: Type::I32,
            value: Some(0),
            mutable: true,
        });
        let m = module.memories.push(MemoryData {
            initial_pages: size,
            maximum_pages: None,
            segments: vec![],
        });
        return Stack { mem: m, glob: g };
    }
    pub fn pop(&self, f: &mut FunctionBody, k: Block, t: Type) -> Value {
        let i32 = f.type_pool.from_iter(vec![Type::I32].into_iter());
        let empty = f.arg_pool.from_iter(std::iter::empty());
        let ts = f.type_pool.from_iter(vec![t].into_iter());
        let size = f.add_value(waffle::ValueDef::Operator(
            Operator::I32Const { value: 8 },
            empty,
            i32,
        ));
        let te = f.type_pool.from_iter(std::iter::empty());
        let l = f.add_value(waffle::ValueDef::Operator(
            Operator::GlobalGet {
                global_index: self.glob,
            },
            empty,
            i32,
        ));
        f.append_to_block(k, l);
        let m = f.arg_pool.from_iter(vec![l, size].into_iter());
        let mi = f.add_value(waffle::ValueDef::Operator(Operator::I32Sub, m, i32));
        f.append_to_block(k, mi);
        let m = f.arg_pool.from_iter(vec![mi].into_iter());
        let n = f.add_value(waffle::ValueDef::Operator(
            match t {
                Type::I32 => Operator::I32Load {
                    memory: MemoryArg {
                        align: 2,
                        offset: 0,
                        memory: self.mem,
                    },
                },
                Type::I64 => Operator::I64Load {
                    memory: MemoryArg {
                        align: 3,
                        offset: 0,
                        memory: self.mem,
                    },
                },
                Type::F32 => Operator::F32Load {
                    memory: MemoryArg {
                        align: 2,
                        offset: 0,
                        memory: self.mem,
                    },
                },
                Type::F64 => Operator::F64Load {
                    memory: MemoryArg {
                        align: 3,
                        offset: 0,
                        memory: self.mem,
                    },
                },
                Type::V128 => todo!(),
                Type::FuncRef => todo!(),
            },
            m,
            ts,
        ));
        f.append_to_block(k, n);
        let m = f.arg_pool.from_iter(vec![mi].into_iter());
        let m = f.add_value(waffle::ValueDef::Operator(
            Operator::GlobalSet {
                global_index: self.glob,
            },
            m,
            te,
        ));
        f.append_to_block(k, m);
        return n;
    }
    pub fn push(&self, v: Value, f: &mut FunctionBody, k: Block) {
        let t = f.values[v].ty(&f.type_pool).unwrap();
        let i32 = f.type_pool.from_iter(vec![Type::I32].into_iter());
        let empty = f.arg_pool.from_iter(std::iter::empty());
        let size = f.add_value(waffle::ValueDef::Operator(
            Operator::I32Const { value: 8 },
            empty,
            i32,
        ));
        let te = f.type_pool.from_iter(std::iter::empty());
        let l = f.add_value(waffle::ValueDef::Operator(
            Operator::GlobalGet {
                global_index: self.glob,
            },
            empty,
            i32,
        ));
        f.append_to_block(k, l);
        let m = f.arg_pool.from_iter(vec![l, v].into_iter());
        let m = f.add_value(waffle::ValueDef::Operator(
            match t {
                Type::I32 => Operator::I32Store {
                    memory: MemoryArg {
                        align: 2,
                        offset: 0,
                        memory: self.mem,
                    },
                },
                Type::I64 => Operator::I64Store {
                    memory: MemoryArg {
                        align: 3,
                        offset: 0,
                        memory: self.mem,
                    },
                },
                Type::F32 => Operator::F32Store {
                    memory: MemoryArg {
                        align: 2,
                        offset: 0,
                        memory: self.mem,
                    },
                },
                Type::F64 => Operator::F64Store {
                    memory: MemoryArg {
                        align: 3,
                        offset: 0,
                        memory: self.mem,
                    },
                },
                Type::V128 => todo!(),
                Type::FuncRef => todo!(),
            },
            m,
            te,
        ));
        f.append_to_block(k, m);
        let m = f.arg_pool.from_iter(vec![l, size].into_iter());
        let m = f.add_value(waffle::ValueDef::Operator(Operator::I32Add, m, i32));
        f.append_to_block(k, m);
        let m = f.arg_pool.from_iter(vec![m].into_iter());
        let m = f.add_value(waffle::ValueDef::Operator(
            Operator::GlobalSet {
                global_index: self.glob,
            },
            m,
            te,
        ));
        f.append_to_block(k, m);
    }
}
