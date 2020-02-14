#[derive(Clone, Debug, PartialEq, Eq)]
pub enum Token {
    ID(String),
    INT(i64),
    FLOAT(String),
    STRING_PART(String),
    STRING_QUOTE,
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

#[derive(Clone, Debug)]
pub enum LexState<'a> {
    NormalInString,
    String(usize, std::str::Chars<'a>),
    IndString,
}


%%
%class Lexer
%field Vec<usize> state_stack
%field usize consumed
%result_type Token

[a-zA-Z_][a-zA-Z0-9_'\-]*  {
    return Ok(Token::ID(self.yytext())); }
[0-9]+  {
    return Ok(Token::INT(self.yytext().parse::<i64>().unwrap())); }
[a-zA-Z0-9\._\-\+]*(/[a-zA-Z0-9\._\-\+]+)+/?  {
    return Ok(Token::PATH(self.yytext()));}

"\"" {
        self.state_stack.push(Lexer::STRING);
        self.yybegin(Lexer::STRING);
        return Ok(Token::STRING_QUOTE);
    }
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
"{" {
        self.state_stack.push(Lexer::YYINITIAL);
        return Ok(Token::OPEN_CURLY);
    }
"}" {
        let state = self.state_stack.pop().unwrap();
        self.yybegin(state);
        return Ok(Token::CLOSE_CURLY);
    }

<STRING>([^\"$]|\$[^\"{]|\\)* { return Ok(Token::STRING_PART(self.yytext())); }
<STRING>"\"" { return Ok(Token::STRING_QUOTE); }
<STRING>"${" {
        self.yybegin(Lexer::YYINITIAL);
        return Ok(Token::DOLLAR_CURLY);
    }

" "    /* Skip white space. This comment cannot be omitted. */
\n     /* Skip white space. This comment cannot be omitted. */
%%

pub fn error_state(&self) -> (usize, usize, usize) {
    (self.zz_marked_pos, self.zz_current_pos, self.zz_start_read)
}

// // like yytext but for the string state (different length)
// pub fn yystring(&mut self, close_length: usize) -> Result<String, Error> {
//     let out = if let Some(LexState::String(string_start, start_char)) = self.state_stack.last() {
//         // TODO(tom) - I don't know why I need the - 1 here.
//         let len = self.zz_marked_pos - string_start - close_length - 1;
//         let mut text = String::with_capacity(len);
//         let mut chars = start_char.clone();
//         for _ in 0..len {
//             text.push(match chars.next() { Some(c) => c, _ => break,});
//         }
//         Ok(text)
//     } else {
//         Err((Error::Unmatch))
//     };
//     self.state_stack.pop();
//     self.yybegin(Lexer::YYINITIAL);
//     return out;
// }
