#[derive(Clone, Debug, PartialEq, Eq)]
pub enum Token {
    ID(String),
    INT(i64),
    FLOAT(String),
    STRING_START,
    DOLLAR_CURLY,
    PATH(String),
    // keywords
    IF,
    THEN,
    ELSE,
    ASSERT,
    WITH,
    LET,
    IN,
    REC,
    INHERIT,
    OR_KW,
    ELLIPSIS,
    // operators
    EQ,
    NEQ,
    LEQ,
    GEQ,
    AND,
    LT,
    GT,
    OR,
    IMPL,
    UPDATE,
    CONCAT,
    MINUS,
    PLUS,
    DIVIDE,
    MULTIPLY,
    ASSIGN,
    // other
    COMMA,
    DOT,
    COLON,
    SEMICOLON,
    QUESTIONMARK,
    AT,
    NEGATE,

    OPEN_CURLY,
    CLOSE_CURLY,
    OPEN_PAREN,
    CLOSE_PAREN,
    OPEN_SQUARE,
    CLOSE_SQUARE,

    WHITE_SPACE,
}

pub enum LexState {
    Normal,
    String,
    IndString,
}

%%


%class Lexer
%field Vec<LexState> state_stack
%field usize consumed
%result_type Token
"{"      return Ok(Token::OPEN_CURLY);
"}"      return Ok(Token::CLOSE_CURLY);
"+"      return Ok(Token::PLUS);
[0-9]+   return Ok(Token::INT(self.yytext().parse::<i64>().unwrap()));
" "    /* Skip white space. This comment cannot be omitted. TODO - check state*/
%%
