# 字符串

其它许多语言中的表示方法一样，Monke语言的字符串是用双引号括起来的一个字符序列。它们也是一等公民。

我们还将用中缀操作符“+”来支持字符串连接。
```
>> let firstName = "Thorsten";
>> let lastName = "Ball";
>> let fullName = fn(first, last) { first + " " + last };
>> fullName(firstName, lastName);
Thorsten Ball
```

## 在词法分析器中支持字符串

字符串的结构如下：
```
"<sequence of characters>"
```
首先需要增加Token类型：
```rust,noplaypen
// src/token/token.rs

pub enum TokenType {
// [...]
    STRING,    // string
}
```

测试用例如下：
```rust,noplaypen
// src/lexer/lexer_test.rs

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

测试失败结果如下：

```
thread 'lexer::tests::test_next_token' panicked at 'test[81] - tokentype wrong. expected=STRING, got=ILLEGAL', src/lexer/lexer_test.rs:262:13
```

扩展词法分析器：
```rust,noplaypen
// src/lexer/lexer.rs

    pub fn next_token(&mut self) -> Token {
// [...]
        match self.ch {
// [...]
            b'"' => {
                tok = Token {
                    r#type: TokenType::STRING,
                    literal: String::from(self.read_string()),
                }
            }
// [...]
        }
// [...]
    }

    fn read_string(&mut self) -> &str {
        let position = self.position + 1;
        loop {
            self.read_char();
            if self.ch == b'"' {
                break;
            }
        }
        &self.input[position..self.position]
    }
```
这里不支持转义字符，您可以自行添加。

测试通过！

词法分析器准备好了，该解析器了。

## 解析字符串

首先定义字符串字面量类型：

```rust,noplaypen
// src/ast/ast.rs

#[derive(Debug, Clone)]
pub struct StringLiteral {
    pub token: Token,
    pub value: String,
}
impl NodeTrait for StringLiteral {
    fn string(&self) -> String {
        format!("{}", self.value)
    }
}

pub enum Expression {
// [...]
    StringLiteral(StringLiteral),
}
impl NodeTrait for Expression {
    fn string(&self) -> String {
        match self {
// [...]
            Expression::StringLiteral(string_literal) => string_literal.string(),
        }
    }
}
```

增加测试用例：
```rust,noplaypen
// src/parser/parser_test.rs

#[test]
fn test_string_literal_expression() {
    let input = r#""hello world";"#;

    let l = Lexer::new(String::from(input));
    let mut p = Parser::new(l);
    match p.parse_program() {
        Ok(Program { statements }) => {
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
                    panic!("exp not StringLiteral. got={:?}", expression);
                }
            } else {
                panic!("parse error");
            }
        }
        Err(errors) => panic_with_errors(errors),
    }
}
```

测试失败结果如下：

```
thread 'parser::parser_test::test_string_literal_expression' panicked at 'parser error: no prefix parse function for STRING found
no prefix parse function for SEMICOLON found', src/parser/parser_test.rs:39:5
```

为字符串Token增加前缀解析函数：
```rust,noplaypen
// src/parser/parser.rs

impl Parser {
    pub fn new(l: Lexer) -> Parser {
// [...]
        p.register_prefix(TokenType::FUNCTION, |parser| {
            parser.parse_function_literal()
        });
        p.register_prefix(TokenType::STRING, |parser| parser.parse_string_literal());
// [...]        
    }


    fn parse_string_literal(&self) -> Result<Expression, String> {
        Ok(Expression::StringLiteral(StringLiteral {
            token: self.cur_token.clone(),
            value: self.cur_token.literal.clone(),
        }))
    }
```

测试通过！

解析器准备好了，该扩展对象系统了。

## 字符串求值

因为Rust支持字符串类型，封装一下即可：
```rust,noplaypen
// src/object/object.rs

#[derive(Debug, PartialEq, Eq, Clone)]
pub struct StringObj {
    pub value: String,
}
impl ObjectTrait for StringObj {
    fn get_type(&self) -> &str {
        "STRING"
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
为了防止跟Rust自带的String类型名称冲突，这里使用StringObj。

增加测试用例：
```rust,noplaypen
// src/evaluator/evaluator_test.rs

#[test]
fn test_string_literal() {
    let input = r#""Hello World!""#;
    let evaluated = test_eval(input);
    if let Object::StringObj(StringObj { value }) = evaluated {
        assert!(
            value == "Hello World!",
            "String has wrong value. got={:?}",
            value
        );
    } else {
        panic!("object is not String. got={:?}", evaluated);
    }
}
```

测试失败结果如下：

```
thread 'evaluator::evaluator_test::test_string_literal' panicked at 'object is not String. got=ErrorObj(ErrorObj { message: "Unknown" })', src/evaluator/evaluator_test.rs:309:9
```

扩展求值器非常容易：
```rust,noplaypen
// src/evaluator/evaluator.rs

pub fn eval(node: Node, env: Rc<RefCell<Environment>>) -> Result<Object, String> {
    match node {
// [...]
        Node::Expression(Expression::StringLiteral(StringLiteral { token: _, value })) => {
            Ok(Object::StringObj(StringObj { value: value }))
        }
        _ => Err(String::from("Unknown")),
    }
}
```

测试通过！

在REPL中使用：
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

使用中缀操作符将两个字符串类型的操作数连接起来。

先写测试用例：
```rust,noplaypen
// src/evaluator/evaluator_test.rs

#[test]
fn test_string_concatenation() {
    let input = r#""Hello" + " " + "World!""#;

    let evaluated = test_eval(input);
    if let Object::StringObj(StringObj { value }) = evaluated {
        assert!(
            value == "Hello World!",
            "String has wrong value. got={:?}",
            value
        );
    } else {
        panic!("object is not String. got={:?}", evaluated);
    }
}
```

再增加一个出错情况的测试用例：
```rust,noplaypen
// src/evaluator/evaluator.rs

fn test_error_handling() {
        let tests = [
// [...]
            (r#""Hello" - "World""#, "unknown operator: STRING - STRING"),
        ];
// [...]
```

测试失败结果如下：

```
thread 'evaluator::evaluator_test::test_string_concatenation' panicked at 'object is not String. got=ErrorObj(ErrorObj { message: "unknown operator: STRING + STRING" })', src/evaluator/evaluator_test.rs:326:9
```

在中缀表达式解析时增加针对字符串的处理方法。
```rust,noplaypen
// src/evaluator/evaluator.rs

fn eval_infix_expression(operator: &str, left: Object, right: Object) -> Result<Object, String> {
    if left.get_type() != right.get_type() {
        return Err(format!(
            "type mismatch: {} {} {}",
            left.get_type(),
            operator,
            right.get_type(),
        ));
    }
    if let Object::StringObj(StringObj { value }) = &left {
        let left_val = value;
        if let Object::StringObj(StringObj { value }) = &right {
            let right_val = value;
            return eval_string_infix_expression(operator, left_val, right_val);
        }
    }
// [...]
}

fn eval_string_infix_expression(operator: &str, left: &str, right: &str) -> Result<Object, String> {
    if operator != "+" {
        return Err(format!("unknown operator: STRING {} STRING", operator));
    }

    Ok(Object::StringObj(StringObj {
        value: format!("{}{}", left, right),
    }))
}
```

测试通过！

如果您需要字符串支持其它中缀操作符，例如“==”或“!=”，请添加到eval_string_infix_expression中。

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

由于字符串输出没有输出外边的双引号，所以带有字符串字面量的表达式（例如函数字面量）输出的并不是真实的情况，您可以思考一下如何解决这个问题（本文中没有处理）。

下面考虑增加一些内置函数。
