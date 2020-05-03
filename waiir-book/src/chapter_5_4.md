# 数组

接下来我们要为Monkey语言增加数组数据类型。

初始化一个新数组，绑定到一个名称，访问其中的元素的Monkey代码如下：
```js
>> let myArray = ["Thorsten", "Ball", 28, fn(x) { x * x }]; 
>> myArray[0]
Thorsten
>> myArray[2]
28
>> myArray[3](2); 
4
```
上面的例子中可以看出Monkey的数组元素可以是任何类型，这里是两个字符串，一个整数，一个函数。

这里访问数组元素的时候使用了一种新的操作符，索引操作符。

本节还将为数组增加几个内置函数，如下：
```js
>> let myArray = ["one", "two", "three"]; 
>> len(myArray)
3
>> first(myArray)
one
>> rest(myArray)
[two, three]
>> last(myArray)
three
>> push(myArray, "four") 
[one, two, three, four]
```


## 在词法分析器中支持数组


需要增加两种Token：
```rust,noplaypen
// src/token/token.rs

pub enum TokenType {
// [...]
    LBRACKET,  // [
    RBRACKET,  // ]
}
```

测试用例如下：
```rust,noplaypen
// src/lexer/lexer_test.rs

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

扩展词法分析器：
```rust,noplaypen
// src/lexer/lexer.rs

    pub fn next_token(&mut self) -> Token {
// [...]
            b'[' => tok = new_token(TokenType::LBRACKET, self.ch),
            b']' => tok = new_token(TokenType::RBRACKET, self.ch),
// [...]            
```
测试通过！

## 解析数组字面量

Monkey中的数组字面量是用逗号分隔，用中括号包围的一系列表达式，如下：
```js
[1, 2, 3 + 3, fn(x) { x }, add(2, 2)]
```

定义如下：
```rust,noplaypen
// src/ast/ast.rs

#[derive(Debug, Clone)]
pub struct ArrayLiteral {
    pub token: Token,
    pub elements: Vec<Expression>,
}
impl NodeTrait for ArrayLiteral {
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
    fn string(&self) -> String {
        match self {
// [...]
            Expression::ArrayLiteral(array_literal) => array_literal.string(),
        }
    }
}
```

增加测试用例：
```rust,noplaypen
// src/parser/parser_test.rs

#[test]
fn test_parsing_array_literals() {
    let input = "[1, 2 * 2, 3 + 3]";

    let l = Lexer::new(String::from(input));
    let mut p = Parser::new(l);
    match p.parse_program() {
        Ok(Program { statements }) => {
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
                        &ExpectedType::Ival(2),
                        "*",
                        &ExpectedType::Ival(2),
                    );
                    test_infix_expression(
                        &elements[2],
                        &ExpectedType::Ival(3),
                        "+",
                        &ExpectedType::Ival(3),
                    );
                } else {
                    panic!("exp not ArrayLiteral. got={:?}", expression);
                }
            } else {
                panic!("parse error");
            }
        }
        Err(errors) => panic_with_errors(errors),
    }
}
```

需要为左中括号Token增加前缀解析函数：
```rust,noplaypen
// src/parser/parser.rs

impl Parser {
    pub fn new(l: Lexer) -> Parser {
// [...]
        p.register_prefix(TokenType::STRING, |parser| parser.parse_string_literal());
        p.register_prefix(TokenType::LBRACKET, |parser| parser.parse_array_literal());
// [...]        
    }    

    fn parse_array_literal(&mut self) -> Result<Expression, String> {
        let token = self.cur_token.clone();
        let elements = self.parse_expression_list(TokenType::RBRACKET)?;
        Ok(Expression::ArrayLiteral(ArrayLiteral {
            token: token,
            elements: elements,
        }))
    }
```
测试通过！

## 解析索引操作符表达式

除了支持数组字面量，还需要支持索引操作符表达式，如下：
```js
myArray[0]; 
myArray[1]; 
myArray[2];
```
还有很多更复杂的表示方式：
```js
[1, 2, 3, 4][2];

let myArray = [1, 2, 3, 4];
myArray[2]; 

myArray[2 + 1]; 

returnsArray()[1];
```
索引操作符表达式的结构如下：
```
<expression>[<expression>]
```

定义AST节点：
```rust,noplaypen
// src/ast/ast.rs

#[derive(Debug, Clone)]
pub struct IndexExpression {
    pub token: Token,
    pub left: Box<Expression>,
    pub index: Box<Expression>,
}
impl NodeTrait for IndexExpression {
    fn string(&self) -> String {
        format!("({}[{}])", self.left.string(), self.index.string())
    }
}

pub enum Expression {
// [...]
    IndexExpression(IndexExpression),
}
impl NodeTrait for Expression {
    fn string(&self) -> String {
        match self {
// [...]
            Expression::IndexExpression(index_expr) => index_expr.string(),
        }
    }
}
```

测试用例如下：
```rust,noplaypen
// src/parser/parser_test.rs

#[test]
fn test_parsing_index_expressions() {
    let input = "myArray[1 + 1]";

    let l = Lexer::new(String::from(input));
    let mut p = Parser::new(l);
    match p.parse_program() {
        Ok(Program { statements }) => {
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
                    test_identifier(left, "myArray");
                    test_infix_expression(
                        index,
                        &ExpectedType::Ival(1),
                        "+",
                        &ExpectedType::Ival(1),
                    );
                } else {
                    panic!("exp not IndexExpression. got={:?}", expression);
                }
            } else {
                panic!("parse error");
            }
        }
        Err(errors) => panic_with_errors(errors),
    }
}
```

索引操作符有最高优先级，增加测试用例：
```rust,noplaypen
// src/parser/parser_test.rs

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
测试失败结果如下：
```
thread 'parser::tests::test_parsing_index_expressions' panicked at 'exp not IndexExpression. got=Identifier(Identifier { token: Token { r#type: IDENT, literal: "myArray" }, value: "myArray" })', src/parser/parser_test.rs:1486:21
thread 'parser::tests::test_operator_precedence_parsing' panicked at 'expected="((a * ([1, 2, 3, 4][(b * c)])) * d)", got="(a * [1, 2, 3, 4])([(b * c)] * d)"', src/parser/parser_test.rs:928:13
```

需要为索引表达式中的左中括号增加中缀解析函数：
```rust,noplaypen
// src/parser/parser.rs

impl Parser {
    pub fn new(l: Lexer) -> Parser {
// [...]
        p.register_infix(TokenType::LPAREN, |parser, exp| {
            parser.parse_call_expression(exp)
        });
        p.register_infix(TokenType::LBRACKET, |parser, exp| {
            parser.parse_index_expression(exp)
        });
// [...]        
    }

    fn parse_index_expression(&mut self, left: Expression) -> Result<Expression, String> {
        let token = self.cur_token.clone();
        self.next_token();
        let index = self.parse_expression(Precedence::LOWEST)?;
        self.expect_peek(&TokenType::RBRACKET)?;
        Ok(Expression::IndexExpression(IndexExpression {
            token: token,
            left: Box::new(left),
            index: Box::new(index),
        }))
    }
```
测试结果还是不正确：
```
thread 'parser::tests::test_parsing_index_expressions' panicked at 'exp not IndexExpression. got=Identifier(Identifier { token: Token { r#type: IDENT, literal: "myArray" }, value: "myArray" })', src/parser/parser_test.rs:1510:21
thread 'parser::tests::test_operator_precedence_parsing' panicked at 'expected="((a * ([1, 2, 3, 4][(b * c)])) * d)", got="(a * [1, 2, 3, 4])([(b * c)] * d)"', src/parser/parser_test.rs:952:13
```
是的，需要添加优先级及其映射：
```rust,noplaypen
// src/parser/parser.rs

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

扩展对象系统，增加数组对象：
```rust,noplaypen
// src/object/object.rs

#[derive(Debug, PartialEq, Eq, Clone)]
pub struct Array {
    pub elements: Vec<Object>,
}
impl ObjectTrait for Array {
    fn get_type(&self) -> &str {
        "ARRAY"
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

测试用例如下：
```rust,noplaypen
// src/evaluator/evaluator_test.rs

#[test]
fn test_array_literals() {
    let input = "[1, 2 * 2, 3 + 3]";
    let evaluated = test_eval(input);
    if let Object::Array(Array { elements }) = evaluated {
        assert!(
            elements.len() == 3,
            "array has wrong num of elments. got={}",
            elements.len()
        );

        test_integer_object(elements[0].clone(), 1);
        test_integer_object(elements[1].clone(), 4);
        test_integer_object(elements[2].clone(), 6);
    } else {
        panic!("object is not Array. got={:?}", evaluated);
    }
}
```
对数组字面量求值，返回数组对象：
```rust,noplaypen
// src/evaluator/evaluator.rs

fn eval(node: Node, env: Rc<RefCell<Environment>>) -> Result<Object, String> {
    match node {
// [...]
        Node::Expression(Expression::ArrayLiteral(ArrayLiteral { token: _, elements })) => {
            let elements_obj = eval_expressions(elements, Rc::clone(&env));
            if elements_obj.len() == 1 && is_error(&elements_obj[0]) {
                return Ok(elements_obj[0].clone());
            }
            Ok(Object::Array(Array {
                elements: elements_obj.iter().map(|x| x.clone()).collect(),
            }))
        }
// [...]
    }
}
```
测试通过！

执行cargo run：
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

要想访问数组元素还需要对索引操作符表达式求值。

## 索引操作符表达式求值

测试用例如下：
```rust,noplaypen
// src/evaluator/evaluator_test.rs

#[test]
fn test_array_index_expressions() {
    let tests = vec![
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
当访问的索引超出了数组索引范围，将返回NULL。

测试失败结果如下：
```
thread 'evaluator::evaluator_test::test_array_index_expressions' panicked at 'object is not Integer. got=ErrorObj(ErrorObj { message: "Unknown" })', src/evaluator/evaluator_test.rs:58:9
```

求值时，需要先求值左部，再求值索引，然后才能确定索引表达式的值：
```rust,noplaypen
// src/evaluator/evaluator.rs

fn eval(node: Node, env: Rc<RefCell<Environment>>) -> Result<Object, String> {
    match node {
// [...]
        Node::Expression(Expression::IndexExpression(IndexExpression {
            token: _,
            left,
            index,
        })) => {
            let left_obj = eval(Node::Expression(*left), Rc::clone(&env))?;
            let index_obj = eval(Node::Expression(*index), Rc::clone(&env))?;
            eval_index_expression(left_obj, index_obj)
        }
// [...]
    }
}
```

eval_index_expression定义如下：
```rust,noplaypen
// src/evaluator/evaluator.rs

fn eval_index_expression(left: Object, index: Object) -> Result<Object, String> {
    if let Object::Array(Array { elements }) = &left {
        if let Object::Integer(Integer { value }) = index {
            return eval_array_index_expression(elements, value);
        }
    }
    Err(format!("index operator not supported: {}", left.get_type()))
}
```
其中eval_array_index_expression定义如下：
```rust,noplaypen
// src/evaluator/evaluator.rs

fn eval_array_index_expression(elements: &Vec<Object>, index: i64) -> Result<Object, String> {
    let idx = index;
    let max = elements.len() as i64 - 1;
    if idx < 0 || idx > max {
        return Ok(Object::Null(NULL));
    }
    return Ok(elements[idx as usize].clone());
}
```
测试通过！

执行cargo run：
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
很容易是么？

## 为数组增加内置函数

为了更方便地使用数组，我们还需要增加一些内置函数。在此之前先扩展len函数，支持返回数组长度：

```rust,noplaypen
// src/evaluator/builtins.rs

pub fn get_builtin(name: &str) -> Option<Object> {
    match name {
        "len" => {
            let func: BuiltinFunction = |args| {
                if args.len() != 1 {
                    return Err(format!(
                        "wrong number of arguments. got={}, want=1",
                        args.len()
                    ));
                }
                return match &args[0] {
                    Object::Array(Array { elements }) => Ok(Object::Integer(Integer {
                        value: elements.len() as i64,
                    })),
                    Object::StringObj(StringObj { value }) => Ok(Object::Integer(Integer {
                        value: value.len() as i64,
                    })),
                    _ => Err(format!(
                        "argument to `len` not supported, got {}",
                        args[0].get_type()
                    )),
                };
            };
            Some(Object::Builtin(Builtin { func: func }))
        }
        _ => None,
    }
}
```
使用Rust Vec的len方法就能做到这一点。

增加first内置函数返回数组的第一个元素：
```rust,noplaypen
// src/evaluator/builtins.rs

pub fn get_builtin(name: &str) -> Option<Object> {
    match name {
// [...]        
        "first" => {
            let func: BuiltinFunction = |args| {
                if args.len() != 1 {
                    return Err(format!(
                        "wrong number of arguments. got={}, want=1",
                        args.len()
                    ));
                }
                if let Object::Array(Array { elements }) = &args[0] {
                    if elements.len() > 0 {
                        return Ok(elements[0].clone());
                    }
                    return Ok(Object::Null(NULL));
                } else {
                    return Err(format!(
                        "arguemnt to `first` must be ARRAY, got={:?}",
                        args[0].get_type()
                    ));
                }
            };
            Some(Object::Builtin(Builtin { func: func }))
        }
// [...]        
```
不难！

接下来增加last内置函数，返回数组最后的元素：
```rust,noplaypen
// src/evaluator/builtins.rs

pub fn get_builtin(name: &str) -> Option<Object> {
    match name {
// [...]        
        "last" => {
            let func: BuiltinFunction = |args| {
                if args.len() != 1 {
                    return Err(format!(
                        "wrong number of arguments. got={}, want=1",
                        args.len()
                    ));
                }
                if let Object::Array(Array { elements }) = &args[0] {
                    let length = elements.len();
                    if length > 0 {
                        return Ok(elements[length - 1].clone());
                    }
                    return Ok(Object::Null(NULL));
                } else {
                    return Err(format!(
                        "arguemnt to `last` must be ARRAY, got={:?}",
                        args[0].get_type()
                    ));
                }
            };
            Some(Object::Builtin(Builtin { func: func }))
        }
// [...]                
```
搞定！

再增加rest内置函数返回除了第一个元素之外的数组元素，返回值仍然是个数组：
```rust,noplaypen
// src/evaluator/builtins.rs

pub fn get_builtin(name: &str) -> Option<Object> {
    match name {
// [...]        
        "rest" => {
            let func: BuiltinFunction = |args| {
                if args.len() != 1 {
                    return Err(format!(
                        "wrong number of arguments. got={}, want=1",
                        args.len()
                    ));
                }
                if let Object::Array(Array { elements }) = &args[0] {
                    let length = elements.len();
                    if length > 0 {                        
                        let mut new_vec: Vec<Object> = vec![Object::Null(NULL); length - 1];
                        new_vec.clone_from_slice(&elements[1..length]);
                        return Ok(Object::Array(Array { elements: new_vec }));
                    }
                    return Ok(Object::Null(NULL));
                } else {
                    return Err(format!(
                        "arguemnt to `rest` must be ARRAY, got={:?}",
                        args[0].get_type()
                    ));
                }
            };
            Some(Object::Builtin(Builtin { func: func }))
        }
// [...]                
```
使用Rust clone_from_slice方法即可！

执行cargo run：
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
注意这里返回的是新创建的数组。

下面实现push函数，使用示例如下：
```js
>> let a = [1, 2, 3, 4]; 
>> let b = push(a, 5); 
>> a
[1, 2, 3, 4]
>> b
[1, 2, 3, 4, 5]
```

实现代码如下：
```rust,noplaypen
// src/evaluator/builtins.rs

pub fn get_builtin(name: &str) -> Option<Object> {
    match name {
// [...]        
        "push" => {
            let func: BuiltinFunction = |args| {
                if args.len() != 2 {
                    return Err(format!(
                        "wrong number of arguments. got={}, want=2",
                        args.len()
                    ));
                }
                if let Object::Array(Array { elements }) = &args[0] {
                    let mut new_elements = elements.to_vec();
                    new_elements.push(args[1].clone());
                    return Ok(Object::Array(Array {
                        elements: new_elements,
                    }));
                } else {
                    return Err(format!(
                        "arguemnt to `push` must be ARRAY, got={:?}",
                        args[0].get_type()
                    ));
                }
            };
            Some(Object::Builtin(Builtin { func: func }))
        }
// [...]  
```


## 测试驱动的数组

使用数组字面量、索引操作符和一些内置函数，我们就可以做很多事情。

先编写一个map函数：
```js
let map = fn(arr, f) {
    let iter = fn(arr, accumulated) {
        if (len(arr) == 0) { 
            accumulated
        } else {
            iter(rest(arr), push(accumulated, f(first(arr))));
        } 
    };
    iter(arr, []); 
};
```
使用时是这样的：
```js
>> let a = [1, 2, 3, 4];
>> let double = fn(x) { x * 2 }; 
>> map(a, double);
[2, 4, 6, 8]
```

很神奇？我们还可以编写一个reduce函数：
```js
let reduce = fn(arr, initial, f) { 
    let iter = fn(arr, result) {
        if (len(arr) == 0) { 
            result
        } else {
            iter(rest(arr), f(result, first(arr)));
        } 
    };
    iter(arr, initial); 
};
```

我们再定义一个sum函数：
```js
let sum = fn(arr) {
    reduce(arr, 0, fn(initial, el) { initial + el });
};
```
看看使用效果：
```js
>> sum([1, 2, 3, 4, 5]); 
15
```

了不起！支持map和reduce了！

接下来我们再加入一种数据结构。