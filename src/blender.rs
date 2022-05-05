#[derive(Debug, Default)]
pub struct Dna {
    pub names: Vec<String>,
    pub types: Vec<String>,
    pub types_len: Vec<usize>,
    pub structs: Vec<DnaStruct>,
}
#[derive(Debug, Default)]
pub struct DnaStruct {
    pub name: String,
    pub fields: Vec<DnaField>,
}
#[derive(Debug, Default)]
pub struct DnaField {
    pub ttype: String,
    pub name: String,
}
