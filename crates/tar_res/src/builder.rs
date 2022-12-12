use crate::{object::Object, store::store_object::StoreObject, Result};

pub fn build(source: String) -> Result<Object> {
    let object: StoreObject = rmp_serde::from_slice(&std::fs::read(source)?)?;

    todo!()
}
