
pub enum Pattern<'src> {
    Identifier(&'src str),
    Literal(&'src str),
    Tuple(Vec<Pattern<'src>>),
}
