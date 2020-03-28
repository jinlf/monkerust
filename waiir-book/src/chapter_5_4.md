# 数组

## 在词法分析器中支持数组

```rust,noplaypen
// src/token.rs

pub enum TokenType {
// [...]
    LBRACKET,  // [
    RBRACKET,  // ]
}
```

```rust,noplaypen
// src/lexer_test.rs

fn test_next_token() {
    let input = "
// [...]
[1, 2];
";

    let tests = [
// [...]
        (TokenType::LBRACKET, "["),
        (TokenType::INT, "1"),
        (TokenType::COMMA, ","),
        (TokenType::INT, "2"),
        (TokenType::RBRACKET, "]"),
        (TokenType::SEMICOLON, ";"),
        (TokenType::EOF, ""),
    ];
```

```rust,noplaypen
// src/lexer.rs

    pub fn next_token(&mut self) -> Token {
// [...]
            b'[' => tok = new_token(TokenType::LBRACKET, self.ch),
            b']' => tok = new_token(TokenType::RBRACKET, self.ch),
// [...]            
```
测试通过！

## 解析数组字面量

```rust,noplaypen
// src/ast.rs

#[derive(Debug, Clone)]
pub struct ArrayLiteral {
    pub token: Token,
    pub elements: Vec<Expression>,
}
impl NodeTrait for ArrayLiteral {
    fn token_literal(&self) -> String {
        self.token.literal.clone()
    }
    fn string(&self) -> String {
        format!(
            "[{}]",
            self.elements
                .iter()
                .map(|x| x.string())
                .collect::<Vec<String>>()
                .join(", ")
        )
    }
}

pub enum Expression {
// [...]
    ArrayLiteral(ArrayLiteral),
}
impl NodeTrait for Expression {
    fn token_literal(&self) -> String {
        match self {
// [...]
            Expression::ArrayLiteral(array_literal) => array_literal.token_literal(),
        }
    }
    fn string(&self) -> String {
        match self {
// [...]
            Expression::ArrayLiteral(array_literal) => array_literal.string(),
        }
    }
}
```

```rust,noplaypen
// src/parser_test.rs

#[test]
fn test_parsing_array_literals() {
    let input = "[1, 2 * 2, 3 + 3]";
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
            if let Expression::ArrayLiteral(ArrayLiteral { token: _, elements }) = expression {
                assert!(
                    elements.len() == 3,
                    "len(array.elements) not 3. got={}",
                    elements.len()
                );
                test_integer_literal(&elements[0], 1);
                test_infix_expression(
                    &elements[1],
                    &*Box::new(2 as i64),
                    String::from("*"),
                    &*Box::new(2 as i64),
                );
                test_infix_expression(
                    &elements[2],
                    &*Box::new(3 as i64),
                    String::from("+"),
                    &*Box::new(3 as i64),
                );
            } else {
                assert!(false, "exp not ArrayLiteral. got={:?}", expression);
            }
        } else {
            assert!(false, "parse error");
        }
    } else {
        assert!(false, "parse error");
    }
}
```

```rust,noplaypen
// src/parser.rs

    fn parse_expression(&mut self, precedence: Precedence) -> Option<Expression> {
        let mut left_exp: Option<Expression>;
        let tk_type = self.cur_token.tk_type.clone();
        match tk_type {
// [...]
            TokenType::LBRACKET => left_exp = self.parse_array_literal(),
// [...]
        }
// [...]
    }        

    fn parse_array_literal(&mut self) -> Option<Expression> {
        let token = self.cur_token.clone();
        let elements = self.parse_expression_list(TokenType::RBRACKET);
        if elements.is_none() {
            return None;
        }

        Some(Expression::ArrayLiteral(ArrayLiteral {
            token: token,
            elements: elements.unwrap(),
        }))
    }

    fn parse_expression_list(&mut self, end: TokenType) -> Option<Vec<Expression>> {
        let mut list: Vec<Expression> = Vec::new();

        if self.peek_token_is(end.clone()) {
            self.next_token();
            return Some(list);
        }

        self.next_token();
        let mut expr = self.parse_expression(Precedence::LOWEST);
        if expr.is_none() {
            return None;
        }
        list.push(expr.unwrap());

        while self.peek_token_is(TokenType::COMMA) {
            self.next_token();
            self.next_token();
            expr = self.parse_expression(Precedence::LOWEST);
            if expr.is_none() {
                return None;
            }
            list.push(expr.unwrap());
        }

        if !self.expect_peek(end) {
            return None;
        }
        Some(list)
    }
```
测试通过！

顺便改一下
```rust,noplaypen
// src/parser.rs

    fn parse_call_expression(&mut self, function: Expression) -> Option<Expression> {
        let token = self.cur_token.clone();
        let arguements = self.parse_expression_list(TokenType::RPAREN);
        if arguements.is_none() {
            return None;
        }

        Some(Expression::CallExpression(CallExpression {
            token: token,
            function: Box::new(function),
            arguments: arguements.unwrap(),
        }))
    }
```

parse_call_arguments就作废了！

## 解析索引操作符表达式

```rust,noplaypen
// src/ast.rs

#[derive(Debug, Clone)]
pub struct IndexExpression {
    pub token: Token,
    pub left: Box<Expression>,
    pub index: Box<Expression>,
}
impl NodeTrait for IndexExpression {
    fn token_literal(&self) -> String {
        self.token.literal.clone()
    }
    fn string(&self) -> String {
        format!("({}[{}])", self.left.string(), self.index.string())
    }
}

pub enum Expression {
// [...]
    IndexExpression(IndexExpression),
}
impl NodeTrait for Expression {
    fn token_literal(&self) -> String {
        match self {
// [...]
            Expression::IndexExpression(index_expr) => index_expr.token_literal(),
        }
    }
    fn string(&self) -> String {
        match self {
// [...]
            Expression::IndexExpression(index_expr) => index_expr.string(),
        }
    }
}
```

```rust,noplaypen
// src/parser_test.rs

#[test]
fn test_parsing_index_expressions() {
    let input = "myArray[1 + 1]";
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
            if let Expression::IndexExpression(IndexExpression {
                token: _,
                left,
                index,
            }) = expression
            {
                test_identifier(left, String::from("myArray"));
                test_infix_expression(
                    index,
                    &*Box::new(1 as i64),
                    String::from("+"),
                    &*Box::new(1 as i64),
                );
            } else {
                assert!(false, "exp not IndexExpression. got={:?}", expression);
            }
        } else {
            assert!(false, "parse error");
        }
    } else {
        assert!(false, "parse error");
    }
}
```

```rust,noplaypen
// src/parser_test.rs

fn test_operator_precedence_parsing() {
    let tests = [
// [...]
        (
            "a * [1, 2, 3, 4][b * c] * d",
            "((a * ([1, 2, 3, 4][(b * c)])) * d)",
        ),
        (
            "add(a * b[2], b[1], 2 * [1, 2][1])",
            "add((a * (b[2])), (b[1]), (2 * ([1, 2][1])))",
        ),
    ];
// [...]        
```
测试结果：
```
thread 'parser::tests::test_parsing_index_expressions' panicked at 'exp not IndexExpression. got=Identifier(Identifier { token: Token { tk_type: IDENT, literal: "myArray" }, value: "myArray" })', src/parser_test.rs:1486:21
thread 'parser::tests::test_operator_precedence_parsing' panicked at 'expected="((a * ([1, 2, 3, 4][(b * c)])) * d)", got="(a * [1, 2, 3, 4])([(b * c)] * d)"', src/parser_test.rs:928:13
```

```rust,noplaypen
// src/parser.rs

    fn parse_expression(&mut self, precedence: Precedence) -> Option<Expression> {
// [...]
        while !self.peek_token_is(TokenType::SEMICOLON) && precedence < self.peek_precedence() {
            let tk_type = self.peek_token.tk_type.clone();
            match tk_type {
// [...]
                TokenType::LBRACKET => {
                    self.next_token();
                    left_exp = self.parse_index_expression(left_exp.unwrap())
                }
                _ => return left_exp,
            }
        }
        left_exp
    }

    fn parse_index_expression(&mut self, left: Expression) -> Option<Expression> {
        let token = self.cur_token.clone();

        self.next_token();
        let index = self.parse_expression(Precedence::LOWEST);
        if index.is_none() {
            return None;
        }

        if !self.expect_peek(TokenType::RBRACKET) {
            return None;
        }

        Some(Expression::IndexExpression(IndexExpression {
            token: token,
            left: Box::new(left),
            index: Box::new(index.unwrap()),
        }))
    }
```
测试结果还是不正确：
```
thread 'parser::tests::test_parsing_index_expressions' panicked at 'exp not IndexExpression. got=Identifier(Identifier { token: Token { tk_type: IDENT, literal: "myArray" }, value: "myArray" })', src/parser_test.rs:1510:21
thread 'parser::tests::test_operator_precedence_parsing' panicked at 'expected="((a * ([1, 2, 3, 4][(b * c)])) * d)", got="(a * [1, 2, 3, 4])([(b * c)] * d)"', src/parser_test.rs:952:13
```

```rust,noplaypen
// src/parser.rs

pub enum Precedence {
// [...]
    INDEX,       // array[index]
}

fn get_precedence(t: &TokenType) -> Precedence {
    match t {
// [...]
        TokenType::LBRACKET => Precedence::INDEX,
        _ => Precedence::LOWEST,
    }
}
```
测试通过！

## 数组字面量求值

```rust,noplaypen
// src/object.rs

#[derive(Debug, PartialEq, Eq, Clone)]
pub struct Array {
    pub elements: Vec<Object>,
}
impl ObjectTrait for Array {
    fn get_type(&self) -> String {
        String::from("ARRAY")
    }
    fn inspect(&self) -> String {
        format!(
            "[{}]",
            self.elements
                .iter()
                .map(|x| x.inspect())
                .collect::<Vec<String>>()
                .join(", ")
        )
    }
}

pub enum Object {
// [...]
    Array(Array),
}
impl ObjectTrait for Object {
    fn get_type(&self) -> String {
        match self {
// [...]
            Object::Array(a) => a.get_type(),
        }
    }
    fn inspect(&self) -> String {
        match self {
// [...]
            Object::Array(a) => a.inspect(),
        }
    }
}
```

```rust,noplaypen
// src/evaluator_test.rs

#[test]
fn test_array_literals() {
    let input = "[1, 2 * 2, 3 + 3]";
    let evaluated = test_eval(input);
    if let Some(Object::Array(Array { elements })) = evaluated {
        assert!(
            elements.len() == 3,
            "array has wrong num of elments. got={}",
            elements.len()
        );

        test_integer_object(Some(elements[0].clone()), 1);
        test_integer_object(Some(elements[1].clone()), 4);
        test_integer_object(Some(elements[2].clone()), 6);
    } else {
        assert!(false, "object is not Array. got={:?}", evaluated);
    }
}
```

```rust,noplaypen
// src/evaluator.rs

pub fn eval(node: Node, env: Rc<RefCell<Environment>>) -> Option<Object> {
    match node {
// [...]
        Node::Expression(Expression::ArrayLiteral(ArrayLiteral { token: _, elements })) => {
            let elements_obj = eval_expressions(elements, Rc::clone(&env));
            if elements_obj.len() == 1 && is_error(&elements_obj[0]) {
                return elements_obj[0].clone();
            }
            Some(Object::Array(Array {
                elements: elements_obj
                    .iter()
                    .filter(|x| x.is_some())
                    .map(|x| x.as_ref().unwrap().clone())
                    .collect(),
            }))
        }
        _ => None,
    }
}
```
测试通过！

用cargo run执行
```
Hello, This is the Monkey programming language!
Feel free to type in commands
>> [1, 2, 3, 4]
[1, 2, 3, 4]
>> let double = fn(x) { x * 2 };
fn(x) {
(x * 2)
}
>> [1, double(2), 3 * 3, 4 - 3];
[1, 4, 9, 1]
>>
```

## 索引操作符求值

```rust,noplaypen
// src/evaluator_test.rs

#[test]
fn test_array_index_expressions() {
    let tests: [(&str, Object); 10] = [
        ("[1, 2, 3][0]", Object::Integer(Integer { value: 1 })),
        ("[1, 2, 3][1]", Object::Integer(Integer { value: 2 })),
        ("[1, 2, 3][2]", Object::Integer(Integer { value: 3 })),
        ("let i = 0; [1][i];", Object::Integer(Integer { value: 1 })),
        ("[1, 2, 3][1 + 1]", Object::Integer(Integer { value: 3 })),
        (
            "let myArray = [1, 2, 3]; myArray[2];",
            Object::Integer(Integer { value: 3 }),
        ),
        (
            "let myArray = [1, 2, 3]; myArray[0] + myArray[1] + myArray[2];",
            Object::Integer(Integer { value: 6 }),
        ),
        (
            "let myArray = [1, 2, 3]; let i = myArray[0]; myArray[i]",
            Object::Integer(Integer { value: 2 }),
        ),
        ("[1, 2, 3][3]", Object::Null(NULL)),
        ("[1, 2, 3][-1]", Object::Null(NULL)),
    ];

    for tt in tests.iter() {
        let evaluated = test_eval(tt.0);
        if let Object::Integer(Integer { value }) = tt.1 {
            test_integer_object(evaluated, value);
        } else {
            test_null_object(evaluated);
        }
    }
}
```
测试结果
```
thread 'evaluator::tests::test_array_index_expressions' panicked at 'object is not Integer. got=None', src/evaluator_test.rs:477:13
```

```rust,noplaypen
// src/evaluator.rs

pub fn eval(node: Node, env: Rc<RefCell<Environment>>) -> Option<Object> {
    match node {
// [...]
        Node::Expression(Expression::IndexExpression(IndexExpression {
            token: _,
            left,
            index,
        })) => {
            let left_obj = eval(Node::Expression(*left), Rc::clone(&env));
            if is_error(&left_obj) {
                return left_obj;
            }
            if left_obj.is_none() {
                return None;
            }
            let index_obj = eval(Node::Expression(*index), Rc::clone(&env));
            if is_error(&index_obj) {
                return index_obj;
            }
            if index_obj.is_none() {
                return None;
            }
            eval_index_expression(left_obj.unwrap(), index_obj.unwrap())
        }
        _ => None,
    }
}
```

```rust,noplaypen
// src/evaluator.rs

fn eval_index_expression(left: Object, index: Object) -> Option<Object> {
    if let Object::Array(_) = left {
        if let Object::Integer(_) = index {
            return eval_array_index_expression(left, index);
        }
    }
    new_error(format!("index operator not supported: {}", left.get_type()))
}
```

```rust,noplaypen
// src/evaluator.rs

fn eval_array_index_expression(array: Object, index: Object) -> Option<Object> {
    if let Object::Array(Array { elements }) = array {
        if let Object::Integer(Integer { value }) = index {
            let idx = value;
            let max = elements.len() as i64 - 1;
            if idx < 0 || idx > max {
                return Some(Object::Null(NULL));
            }
            return Some(elements[idx as usize].clone());
        }
    }
    None
}
```
测试通过！

用cargo run执行
```
Hello, This is the Monkey programming language!
Feel free to type in commands
>> let a = [1, 2 * 2, 10 - 5, 8 / 2];
[1, 4, 5, 4]
>> a[0]
1
>> a[1]
4
>> a[5 - 3]
5
>> a[99]
null
>> 
```

## 为数组增加内置函数

```rust,noplaypen
// src/builtins.rs

pub fn get_builtin(name: &str) -> Option<Object> {
    match name {
        "len" => {
            let func: BuiltinFunction = |args| {
                if args.len() != 1 {
                    return new_error(format!(
                        "wrong number of arguments. got={}, want=1",
                        args.len()
                    ));
                }
                return match &args[0] {
                    Some(Object::Array(Array { elements })) => Some(Object::Integer(Integer {
                        value: elements.len() as i64,
                    })),
                    Some(Object::StringObj(StringObj { value })) => {
                        Some(Object::Integer(Integer {
                            value: value.len() as i64,
                        }))
                    }
                    _ => new_error(format!(
                        "argument to `len` not supported, got {}",
                        get_type(&args[0])
                    )),
                };
            };
            Some(Object::Builtin(Builtin { func: func }))
        }
        _ => None,
    }
```

再增加
```rust,noplaypen
// src/builtins.rs

pub fn get_builtin(name: &str) -> Option<Object> {
    match name {
// [...]        
        "first" => {
            let func: BuiltinFunction = |args| {
                if args.len() != 1 {
                    return new_error(format!(
                        "wrong number of arguments. got={}, want=1",
                        args.len()
                    ));
                }
                if let Some(Object::Array(Array { elements })) = &args[0] {
                    if elements.len() > 0 {
                        return Some(elements[0].clone());
                    }
                    return Some(Object::Null(NULL));
                } else {
                    return new_error(format!(
                        "arguemnt to `first` must be ARRAY, got={:?}",
                        get_type(&args[0])
                    ));
                }
            };
            Some(Object::Builtin(Builtin { func: func }))
        }
// [...]        
```


```rust,noplaypen
// src/builtins.rs

pub fn get_builtin(name: &str) -> Option<Object> {
    match name {
// [...]        
        "last" => {
            let func: BuiltinFunction = |args| {
                if args.len() != 1 {
                    return new_error(format!(
                        "wrong number of arguments. got={}, want=1",
                        args.len()
                    ));
                }
                if let Some(Object::Array(Array { elements })) = &args[0] {
                    let length = elements.len();
                    if length > 0 {
                        return Some(elements[length - 1].clone());
                    }
                    return Some(Object::Null(NULL));
                } else {
                    return new_error(format!(
                        "arguemnt to `last` must be ARRAY, got={:?}",
                        get_type(&args[0])
                    ));
                }
            };
            Some(Object::Builtin(Builtin { func: func }))
        }
// [...]                
```

```rust,noplaypen
// src/builtins.rs

pub fn get_builtin(name: &str) -> Option<Object> {
    match name {
// [...]        
        "rest" => {
            let func: BuiltinFunction = |args| {
                if args.len() != 1 {
                    return new_error(format!(
                        "wrong number of arguments. got={}, want=1",
                        args.len()
                    ));
                }
                if let Some(Object::Array(Array { elements })) = &args[0] {
                    let length = elements.len();
                    if length > 0 {
                        return Some(Object::Array(Array {
                            elements: elements[1..length].to_vec(),
                        }));
                    }
                    return Some(Object::Null(NULL));
                } else {
                    return new_error(format!(
                        "arguemnt to `rest` must be ARRAY, got={:?}",
                        get_type(&args[0])
                    ));
                }
            };
            Some(Object::Builtin(Builtin { func: func }))
        }
// [...]                
```

用cargo run执行
```
Hello, This is the Monkey programming language!
Feel free to type in commands
>> let a = [1, 2, 3, 4];
[1, 2, 3, 4]
>> rest(a)
[2, 3, 4]
>> rest(rest(a))
[3, 4]
>> rest(rest(rest(a)))
[4]
>> rest(rest(rest(rest(a))))
[]
>> rest(rest(rest(rest(rest(a)))))
null
>> 
```

```rust,noplaypen
// src/builtins.rs

pub fn get_builtin(name: &str) -> Option<Object> {
    match name {
// [...]        
        "push" => {
            let func: BuiltinFunction = |args| {
                if args.len() != 2 {
                    return new_error(format!(
                        "wrong number of arguments. got={}, want=2",
                        args.len()
                    ));
                }
                if let Some(Object::Array(Array { elements })) = &args[0] {
                    let length = elements.len();
                    if length > 0 {
                        let mut new_elements = elements.to_vec();
                        new_elements.push(args[1].as_ref().unwrap().clone());
                        return Some(Object::Array(Array {
                            elements: new_elements,
                        }));
                    }
                    return Some(Object::Null(NULL));
                } else {
                    return new_error(format!(
                        "arguemnt to `push` must be ARRAY, got={:?}",
                        get_type(&args[0])
                    ));
                }
            };
            Some(Object::Builtin(Builtin { func: func }))
        }
// [...]  
```
用cargo run
```
Hello, This is the Monkey programming language!
Feel free to type in commands
>> let a = [1, 2, 3, 4];
[1, 2, 3, 4]
>> let b = push(a, 5);
[1, 2, 3, 4, 5]
>> a
[1, 2, 3, 4]
>> b
[1, 2, 3, 4, 5]
>> 
```

## 测试驱动的数组

