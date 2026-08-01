pub struct Parser<'a> {
    lexer: Lexer<'a>,
    current: Token,
}
impl<'a> Parser<'a> {
    pub fn new(input: &'a str) -> Result<Self, ParseError>;

    fn advance(&mut self);

    fn expect(&mut self, token: Token);

    pub fn parse(&mut self) -> Result<Expr>;
}
