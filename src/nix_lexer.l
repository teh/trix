#[allow(non_camel_case_types)]

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum Token {
    ID(String),
    INT(i64),
    FLOAT(String),
    STRING_PART(String),
    STRING_QUOTE,
    INDENTED_STRING_QUOTE,
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


%%
%class Lexer
%field Vec<usize> state_stack
%field usize consumed
%result_type Token

[0-9]+  {
    return Ok(Token::INT(self.yytext().parse::<i64>().unwrap())); }

"\"" {
        self.state_stack.push(self.yystate());
        self.yybegin(Lexer::STRING);
        return Ok(Token::STRING_QUOTE);
    }
"''" {
        self.state_stack.push(self.yystate());
        self.yybegin(Lexer::INDENTEDSTRING);
        return Ok(Token::INDENTED_STRING_QUOTE);
    }
"or"            return Ok(Token::OR_KW);
","             return Ok(Token::COMMA);
"${" {
        self.state_stack.push(self.yystate());
        self.yybegin(Lexer::YYINITIAL);
        return Ok(Token::DOLLAR_CURLY);
    }
if              return Ok(Token::IF);
then            return Ok(Token::THEN);
else            return Ok(Token::ELSE);
assert          return Ok(Token::ASSERT);
with            return Ok(Token::WITH);
let             return Ok(Token::LET);
in              return Ok(Token::IN);
rec             return Ok(Token::REC);
inherit         return Ok(Token::INHERIT);
[a-zA-Z_][a-zA-Z0-9_'\-]*  {
    return Ok(Token::ID(self.yytext())); }
[a-zA-Z0-9\._\-\+]*(/[a-zA-Z0-9\._\-\+]+)+/?  {
    return Ok(Token::PATH(self.yytext()));}
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
        self.state_stack.push(self.yystate());
        return Ok(Token::OPEN_CURLY);
    }
"}" {
        let state = self.state_stack.pop().unwrap();
        self.yybegin(state);
        return Ok(Token::CLOSE_CURLY);
    }

<STRINGDOLLAR>"" {
        let state = self.state_stack.pop().unwrap(); // double-pop
        let state = self.state_stack.pop().unwrap();
        self.yybegin(state);
        return Ok(Token::STRING_QUOTE);
    }

<STRING>([^\"$]|\$[^\"{]|\\.|\\\n|\$\\.)* {
        return Ok(Token::STRING_PART(self.yytext()));
    }
<STRING>"\"" {
        let state = self.state_stack.pop().unwrap();
        self.yybegin(state);
        return Ok(Token::STRING_QUOTE);
    }
<STRING>"${" {
        self.state_stack.push(self.yystate());
        self.yybegin(Lexer::YYINITIAL);
        return Ok(Token::DOLLAR_CURLY);
    }
<STRING>"$\"" {
        // If a string ends in `$` we don't really have a way capture that
        // with the regex to parse strings, instead we match the entire
        // end in one go, and then use the helper-state STRINGDOLLAR to return
        // more than one token.
        self.state_stack.push(self.yystate());
        self.yybegin(Lexer::STRINGDOLLAR);
        return Ok(Token::STRING_PART("$".to_string()));
    }

<INDENTEDSTRING>([^'$]|\$[^'{]|'[^'$])+ { return Ok(Token::STRING_PART(self.yytext())); }
<INDENTEDSTRING>''\$ { return Ok(Token::STRING_PART("$".to_string())); }
<INDENTEDSTRING>\$ { return Ok(Token::STRING_PART("$".to_string())); }
<INDENTEDSTRING>''' { return Ok(Token::STRING_PART("''".to_string())); }
<INDENTEDSTRING>''\\. { return Ok(Token::STRING_PART(self.yytext())); }
<INDENTEDSTRING>''\\\n { return Ok(Token::STRING_PART(self.yytext())); }
<INDENTEDSTRING>''\$ { return Ok(Token::STRING_PART(self.yytext())); }
<INDENTEDSTRING>' { return Ok(Token::STRING_PART(self.yytext())); }
<INDENTEDSTRING>'' {
        let state = self.state_stack.pop().unwrap();
        self.yybegin(state);
        return Ok(Token::INDENTED_STRING_QUOTE);
    }
<INDENTEDSTRING>"${" {
        self.state_stack.push(self.yystate());
        self.yybegin(Lexer::YYINITIAL);
        return Ok(Token::DOLLAR_CURLY);
    }


#.*$                       /* single-line comments */
/\*([^\*]|\*+[^\*/])*\*+/  /* long comments */

" "    /* comment needed, do not remove */
\n     /* comment needed, do not remove */
\t     /* comment needed, do not remove */
%%

pub fn error_state(&self) -> (usize, usize, usize, usize) {
    (self.zz_lineno, self.zz_start_read, self.zz_current_pos, self.zz_marked_pos)
}
