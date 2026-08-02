use tinyexpr_rs::lexer::Lexer;
use tinyexpr_rs::token::Token;

#[test]
fn tokenize_simple_expression() {
    let mut lexer = Lexer::new("2 + sin(3)");

    assert_eq!(lexer.next_token().unwrap(), Token::Number(2.0));
    assert_eq!(lexer.next_token().unwrap(), Token::Plus);
    assert_eq!(
        lexer.next_token().unwrap(),
        Token::Identifier("sin".to_string())
    );
    assert_eq!(lexer.next_token().unwrap(), Token::OpenParen);
    assert_eq!(lexer.next_token().unwrap(), Token::Number(3.0));
    assert_eq!(lexer.next_token().unwrap(), Token::CloseParen);
    assert_eq!(lexer.next_token().unwrap(), Token::End);
}
