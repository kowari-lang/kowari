#[derive(Debug, Clone)]
pub enum Literal<'src> {
    Integer(&'src str),
    Float(&'src str),
    String(&'src str),
    Char(&'src str),
    Boolean(bool),
    Path(&'src str),
}
