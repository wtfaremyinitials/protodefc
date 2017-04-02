use ::errors::*;
use ::ir::{TypeVariant, TypeData, TypeContainer};
use ::ir::variant::{SimpleScalarVariant, ContainerVariant, ArrayVariant, UnionVariant};
use super::*;
use super::utils::*;
use super::container_utils::*;

pub fn generate_serialize(typ: TypeContainer) -> Result<Block> {
    let typ_inner = typ.borrow();
    codegen_for_type(&*typ_inner)
        .serialize(&typ_inner.data)
}

pub trait BaseSerialize: TypeVariant {
    fn serialize(&self, data: &TypeData) -> Result<Block>;
}

impl BaseSerialize for SimpleScalarVariant {
    fn serialize(&self, data: &TypeData) -> Result<Block> {
        Ok(Block(vec![
            Operation::Assign {
                name: "offset".to_owned().into(),
                value: Expr::TypeCall {
                    typ: CallType::Serialize,
                    type_name: data.name.clone().into(),
                    input: input_for(data).into(),
                },
            }
        ]))
    }
}

impl BaseSerialize for ContainerVariant {
    fn serialize(&self, data: &TypeData) -> Result<Block> {
        let mut ops: Vec<Operation> = Vec::new();

        for (idx, field) in self.fields.iter().enumerate() {
            let child_typ = field.child.upgrade().unwrap();

            build_var_accessor(self, data, &mut ops, idx)?;
            ops.push(Operation::Block(generate_serialize(child_typ)?));
        }

        Ok(Block(ops))
    }
}

impl BaseSerialize for ArrayVariant {
    fn serialize(&self, data: &TypeData) -> Result<Block> {
        let mut ops: Vec<Operation> = Vec::new();

        let ident = data.ident.unwrap();
        let index_var = format!("array_{}_index", ident);

        let child_input_var = input_for_type(self.child.upgrade().unwrap());

        ops.push(Operation::ForEachArray {
            array: input_for(data).into(),
            index: index_var.clone().into(),
            typ: child_input_var.clone().into(),
            block: generate_serialize(self.child.upgrade().unwrap())?,
        });

        Ok(Block(ops))
    }
}

impl BaseSerialize for UnionVariant {
    fn serialize(&self, data: &TypeData) -> Result<Block> {
        let mut ops: Vec<Operation> = Vec::new();

        let cases: Result<Vec<UnionTagCase>> = self.cases.iter().map(|case| {
            let child_rc = case.child.upgrade().unwrap();
            let child_inner = child_rc.borrow();

            let mut i_ops: Vec<Operation> = Vec::new();

            let inner = generate_serialize(child_rc.clone())?;
            i_ops.push(Operation::Block(inner));

            Ok(UnionTagCase {
                variant_name: case.case_name.clone(),
                variant_var: Some(
                    input_for(&child_inner.data).into()),
                block: Block(i_ops),
            })
        }).collect();

        ops.push(Operation::MapValue {
            input: input_for(data).into(),
            output: "".to_owned().into(),
            operation: MapOperation::UnionTagToExpr(cases?),
        });

        Ok(Block(ops))
    }
}