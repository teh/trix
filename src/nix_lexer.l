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

[a-zA-Z_][a-zA-Z0-9_'\-]*                           return Ok(Token::ID(self.yytext()));
[0-9]+                                              return Ok(Token::INT(self.yytext().parse::<i64>().unwrap()));
[a-zA-Z0-9\._\-\+]*(/[a-zA-Z0-9\._\-\+]+)+/?    return Ok(Token::PATH(self.yytext()));

"\""            return Ok(Token::STRING_START);
"or"            return Ok(Token::OR_KW);
","             return Ok(Token::COMMA);
"${"            return Ok(Token::DOLLAR_CURLY);
if              return Ok(Token::IF);
then            return Ok(Token::THEN);
else            return Ok(Token::ELSE);
assert          return Ok(Token::ASSERT);
with            return Ok(Token::WITH);
let             return Ok(Token::LET);
in              return Ok(Token::IN);
rec             return Ok(Token::REC);
inherit         return Ok(Token::INHERIT);
"..."           return Ok(Token::ELLIPSIS);
","             return Ok(Token::COMMA);
"."             return Ok(Token::DOT);
":"             return Ok(Token::COLON);
";"             return Ok(Token::SEMICOLON);
"?"             return Ok(Token::QUESTIONMARK);
"@"             return Ok(Token::AT);
"="             return Ok(Token::ASSIGN);
"=="            return Ok(Token::EQ);
"!="            return Ok(Token::NEQ);
"<="            return Ok(Token::LEQ);
">="            return Ok(Token::GEQ);
"<"             return Ok(Token::LT);
">"             return Ok(Token::GT);
"&&"            return Ok(Token::AND);
"||"            return Ok(Token::OR);
"->"            return Ok(Token::IMPL);
"//"            return Ok(Token::UPDATE);
"++"            return Ok(Token::CONCAT);
"-"             return Ok(Token::MINUS);
"+"             return Ok(Token::PLUS);
"/"             return Ok(Token::DIVIDE);
"*"             return Ok(Token::MULTIPLY);
"!"             return Ok(Token::NEGATE);
"("             return Ok(Token::OPEN_PAREN);
")"             return Ok(Token::CLOSE_PAREN);
"["             return Ok(Token::OPEN_SQUARE);
"]"             return Ok(Token::CLOSE_SQUARE);
"{"             return Ok(Token::OPEN_CURLY);
"}"             return Ok(Token::CLOSE_CURLY);

[0-9]+   return Ok(Token::INT(self.yytext().parse::<i64>().unwrap()));
" "    /* Skip white space. This comment cannot be omitted. TODO - check state*/
%%
