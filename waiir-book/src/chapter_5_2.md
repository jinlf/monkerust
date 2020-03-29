# 字符串

## 在词法分析器中支持字符串

```rust,noplaypen
// src/token.rs

pub enum TokenType {
// [...]
    STRING,    // string
}
```

```rust,noplaypen
// src/lexer_test.rs

    fn test_next_token() {
        let input = "
// [...]
10 == 10;
10 != 9;
\"foobar\"
\"foo bar\"
";
// [...]
            (TokenType::STRING, "foobar"),
            (TokenType::STRING, "foo bar"),
            (TokenType::EOF, ""),
        ];
// [...]
```

测试结果：

```
thread 'lexer::tests::test_next_token' panicked at 'test[81] - tokentype wrong. expected=STRING, got=ILLEGAL', src/lexer_test.rs:262:13
```

```rust,noplaypen
// src/lexer.rs

    pub fn next_token(&mut self) -> Token {
// [...]
        match self.ch {
// [...]
            b'"' => {
                tok = Token {
                    tk_type: TokenType::STRING,
                    literal: self.read_string(),
                }
            }
// [...]
        }
// [...]
    }

    fn read_string(&mut self) -> String {
        let position = self.position + 1;
        loop {
            self.read_char();
            if self.ch == b'"' {
                break;
            }
        }
        String::from(&self.input[position..self.position])
    }
```

测试通过！

## 解析字符串

```rust,noplaypen
// src/ast.rs

#[derive(Debug, Clone)]
pub struct StringLiteral {
    pub token: Token,
    pub value: String,
}
impl NodeTrait for StringLiteral {
    fn token_literal(&self) -> String {
        self.token.literal.clone()
    }
    fn string(&self) -> String {
        format!("{}", self.value)
    }
}

pub enum Expression {
// [...]
    StringLiteral(StringLiteral),
}
impl NodeTrait for Expression {
    fn token_literal(&self) -> String {
        match self {
// [...]
            Expression::StringLiteral(string_literal) => string_literal.token_literal(),
        }
    }
    fn string(&self) -> String {
        match self {
// [...]
            Expression::StringLiteral(string_literal) => string_literal.string(),
        }
    }
}
```

```rust,noplaypen
// src/parser_test.rs

#[test]
fn test_string_literal_expression() {
    let input = r#""hello world";"#;
    let l = Lexer::new(input);
    let mut p = Parser::new(l);
    let program = p.parse_program();
    check_parser_errors(&mut p);

    if let Some(Program { statements }) = program {
        if let Statement::ExpressionStatement(ExpressionStatement {
            token: _,
            expression,
        }) = &statements[0]
        {
            if let Expression::StringLiteral(StringLiteral { token: _, value }) = expression {
                assert!(
                    value == "hello world",
                    "literal.value not {}. got={}",
                    "hello world",
                    value
                );
            } else {
                assert!(false, "exp not StringLiteral. got={:?}", expression);
            }
        } else {
            assert!(false, "parse error");
        }
    } else {
        assert!(false, "parse error");
    }
}
```

测试结果：

```
thread 'parser::tests::test_string_literal_expression' panicked at 'parser has 2 errors
parser error: "no prefix parse function for STRING found"
parser error: "no prefix parse function for SEMICOLON found"
', src/parser_test.rs:549:9
```

```rust,noplaypen
// src/parser.rs

    fn parse_expression(&mut self, precedence: Precedence) -> Option<Expression> {
        let mut left_exp: Option<Expression>;
        let tk_type = self.cur_token.tk_type.clone();
        match tk_type {
// [...]
            TokenType::STRING => left_exp = self.parse_string_literal(),
// [...]
        }
// [...]
    }

    fn parse_string_literal(&self) -> Option<Expression> {
        Some(Expression::StringLiteral(StringLiteral {
            token: self.cur_token.clone(),
            value: self.cur_token.literal.clone(),
        }))
    }
```

测试通过！

## 字符串求值

```rust,noplaypen
// src/object.rs

#[derive(Debug, PartialEq, Eq, Clone)]
pub struct StringObj {
    pub value: String,
}
impl ObjectTrait for StringObj {
    fn get_type(&self) -> String {
        String::from("STRING")
    }
    fn inspect(&self) -> String {
        self.value.clone()
    }
}

pub enum Object {
// [...]
    StringObj(StringObj),
}
impl ObjectTrait for Object {
    fn get_type(&self) -> String {
        match self {
// [...]
            Object::StringObj(s) => s.get_type(),
        }
    }
    fn inspect(&self) -> String {
        match self {
// [...]
            Object::StringObj(s) => s.inspect(),
        }
    }
}
```

```rust,noplaypen
// src/evaluator_test.rs

#[test]
fn test_string_literal() {
    let input = r#""Hello World!""#;
    let evaluated = test_eval(input);
    if let Some(Object::StringObj(StringObj { value })) = evaluated {
        assert!(
            value == "Hello World!",
            "String has wrong value. got={:?}",
            value
        );
    } else {
        assert!(false, "object is not String. got={:?}", evaluated);
    }
}
```

测试结果：

```
thread 'evaluator::tests::test_string_literal' panicked at 'object is not String. got=None', src/evaluator_test.rs:658:13
```

```rust,noplaypen
// src/evaluator.rs

pub fn eval(node: Node, env: Rc<RefCell<Environment>>) -> Option<Object> {
    match node {
// [...]
        Node::Expression(Expression::StringLiteral(StringLiteral { token: _, value })) => {
            Some(Object::StringObj(StringObj { value: value }))
        }
        _ => None,
    }
}
```

测试通过！

```
Hello, This is the Monkey programming language!
Feel free to type in commands
>> "Hello World!"
Hello World!
>> let hello = "Hello there, fellow Monkey users and fans!"
Hello there, fellow Monkey users and fans!
>> hello
Hello there, fellow Monkey users and fans!
>> let giveMeHello = fn() { "Hello!" }
fn() {
Hello!
}
>> giveMeHello()
Hello!
>> "This is amazing!"
This is amazing!
>>
```

## 字符串连接

```rust,noplaypen
// src/evaluator_test.rs

#[test]
fn test_string_concatenation() {
    let input = r#""Hello" + " " + "World!""#;
    let evaluated = test_eval(input);
    if let Some(Object::StringObj(StringObj { value })) = evaluated {
        assert!(
            value == "Hello World!",
            "String has wrong value. got={:?}",
            value
        );
    } else {
        assert!(false, "object is not String. got={:?}", evaluated);
    }
}
```

```rust,noplaypen
// src/evaluator.rs

fn test_error_handling() {
        let tests = [
// [...]
            (r#""Hello" - "World""#, "unknown operator: STRING - STRING"),
        ];
// [...]
```

测试结果：

```
thread 'evaluator::tests::test_string_concatenation' panicked at 'object is not String. got=Some(ErrorObj(ErrorObj { message: "unknown operator: STRING + STRING" }))', src/evaluator_test.rs:677:13
```

```rust,noplaypen
// src/evaluator.rs

fn eval_infix_expression(
    operator: &str,
    left: Option<Object>,
    right: Option<Object>,
) -> Option<Object> {
    if get_type(&left) != get_type(&right) {
        return new_error(format!(
            "type mismatch: {} {} {}",
            get_type(&left),
            operator,
            get_type(&right)
        ));
    }
    if let Some(Object::StringObj(_)) = left {
        if let Some(Object::StringObj(_)) = right {
            return eval_string_infix_expression(operator, &left, &right);
        }
    }
// [...]
}

fn eval_string_infix_expression(
    operator: &str,
    left: &Option<Object>,
    right: &Option<Object>,
) -> Option<Object> {
    if operator != "+" {
        return new_error(format!(
            "unknown operator: {} {} {}",
            get_type(&left),
            operator,
            get_type(&right)
        ));
    }
    if let Some(Object::StringObj(StringObj { value })) = left {
        let left_val = value;
        if let Some(Object::StringObj(StringObj { value })) = right {
            let right_val = value;
            return Some(Object::StringObj(StringObj {
                value: format!("{}{}", left_val, right_val),
            }));
        }
    }
    None
}
```

测试通过！

执行 cargo run

```
Hello, This is the Monkey programming language!
Feel free to type in commands
>> let makeGreeter = fn(greeting) { fn(name) { greeting + " " + name + "!" } };
fn(greeting) {
fn (name) (((greeting +  ) + name) + !)
}
>> let hello = makeGreeter("Hello");
fn(name) {
(((greeting +  ) + name) + !)
}
>> hello("Jerry");
Hello Jerry!
>> let heythere = makeGreeter("Hey there");
fn(name) {
(((greeting +  ) + name) + !)
}
>> heythere("Jerry");
Hey there Jerry!
>> 
```