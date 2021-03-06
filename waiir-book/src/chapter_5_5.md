# 哈希

哈希（hash）在很多语言中的名称不同，如map，哈希map，字典等。其功能是实现键（key）和值（value）映射。

```js
>> let myHash = {"name": "Jimmy", "age": 72, "band": "Led Zeppelin"}; 
>> myHash["name"]
Jimmy
>> myHash["age"]
72
>> myHash["band"] 
Led Zeppelin
```
上面例子中myHash有三个键，都是字符串。我们可以用索引操作符表达式来访问值，只是用到的索引值是字符串。

当然，其它类型的键也是可以的：
```js
>> let myHash = {true: "yes, a boolean", 99: "correct, an integer"}; 
>> myHash[true]
yes, a boolean
>> myHash[99]
correct, an integer
```
表达式作为键也是可以的：
```js
>> myHash[5 > 1] 
yes, a boolean
>> myHash[100 - 1] 
correct, an integer
```

## 哈希字面量的词法分析

哈希字面量的例子如下：
```js
{"name": "Jimmy", "age": 72, "band": "Led Zeppelin"}
```
这里有一个词法分析器不支持的符号“:”，需要添加进Token类型中：

```rust,noplaypne
// src/token/token.rs

pub enum TokenType {
// [...]
    COLON,     // :
}
```

测试用例：
```rust,noplaypen
// src/lexer/lexer_test.rs

fn test_next_token() {
    let input = "
// [...]
{\"foo\": \"bar\"}
";
// [...]
        (TokenType::LBRACE, "{"),
        (TokenType::STRING, "foo"),
        (TokenType::COLON, ":"),
        (TokenType::STRING, "bar"),
        (TokenType::RBRACE, "}"),
        (TokenType::EOF, ""),
    ];
// [...]        
}
```

扩展词法分析器：
```rust,noplaypen
// src/lexer/lexer.rs

    pub fn next_token(&mut self) -> Token {
        let tok: Token;

        self.skip_whitespace();
        
        match self.ch {
            b':' => tok = new_token(TokenType::COLON, self.ch),
// [...]
```
测试通过！

## 解析哈希字面量

哈希字面量的结构如下：
```js
{<expression>: <expression>, <expression>: <expression>, ...}
```
即用逗号分隔用大括号括起来的的键值对列表。

用Rust语言的HashMap来保存这种对应关系，定义如下：
```rust,noplaypen
// src/ast/ast.rs

use std::collections::*;

#[derive(Clone)]
pub struct HashLiteral {
    pub token: Token,
    pub pairs: HashMap<Expression, Expression>,
}
impl NodeTrait for HashLiteral {
    fn string(&self) -> String {
        format!(
            "{{{}}}",
            self.pairs
                .iter()
                .map(|(k, v)| format!("{}:{}", k.string(), v.string()))
                .collect::<Vec<String>>()
                .join(", ")
        )
    }
}
impl std::fmt::Debug for HashLiteral {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.string())
    }
}

pub enum Expression {
// [...]
    HashLiteral(HashLiteral),
}
impl NodeTrait for Expression {
    fn string(&self) -> String {
        match self {
// [...]
            Expression::HashLiteral(hash_literal) => hash_literal.string(),
        }
    }
}
```
测试用例：
```rust,noplaypen
// src/parser/parser_test.rs

use std::collections::*;

#[test]
fn test_parsing_hash_literals_string_keys() {
    let input = r#"{"one": 1, "two": 2, "three": 3}"#;

    let l = Lexer::new(String::from(input));
    let mut p = Parser::new(l);
    match p.parse_program() {
        Ok(Program { statements }) => {
            if let Statement::ExpressionStatement(ExpressionStatement {
                token: _,
                expression,
            }) = &statements[0]
            {
                if let Expression::HashLiteral(HashLiteral { token: _, pairs }) = expression {
                    assert!(
                        pairs.len() == 3,
                        "hash.pairs has wrong length. got={}",
                        pairs.len()
                    );
                    let mut expected: HashMap<String, i64> = HashMap::new();
                    expected.insert(String::from("one"), 1);
                    expected.insert(String::from("two"), 2);
                    expected.insert(String::from("three"), 3);
                    for (key, value) in pairs.iter() {
                        if let Expression::StringLiteral(literal) = key {
                            let expected_value = expected.get(&literal.string());
                            test_integer_literal(value, *expected_value.unwrap());
                        } else {
                            panic!("key is not StringLiteral. got={:?}", key);
                        }
                    }
                } else {
                    panic!("exp is not HashLiteral. got={:?}", expression);
                }
            } else {
                panic!("parse error");
            }
        }
        Err(errors) => panic_with_errors(errors),
    }
}
```
需要能够支持内容为空的哈希，测试用例如下：
```rust,noplaypen
// src/parser/parser_test.rs

#[test]
fn test_parsing_empty_hash_literal() {
    let input = "{}";
    let l = Lexer::new(String::from(input));
    let mut p = Parser::new(l);
    match p.parse_program() {
        Ok(Program { statements }) => {
            if let Statement::ExpressionStatement(ExpressionStatement {
                token: _,
                expression,
            }) = &statements[0]
            {
                if let Expression::HashLiteral(HashLiteral { token: _, pairs }) = expression {
                    assert!(
                        pairs.len() == 0,
                        "hash.pairs has wrong length. got={}",
                        pairs.len()
                    );
                } else {
                    panic!("exp is not HashLiteral. got={:?}", expression);
                }
            } else {
                panic!("parse error");
            }
        }
        Err(errors) => panic_with_errors(errors),
    }
}
```
哈希键和值都可以是表达式，测试用例如下：
```rust,noplaypen
// src/parser/parser_test.rs

#[test]
fn test_parsing_hash_literal_with_expressions() {
    let input = r#"{"one": 0 + 1, "two": 10 - 8, "three": 15 / 5}"#;

    let l = Lexer::new(String::from(input));
    let mut p = Parser::new(l);
    match p.parse_program() {
        Ok(Program { statements }) => {
            if let Statement::ExpressionStatement(ExpressionStatement {
                token: _,
                expression,
            }) = &statements[0]
            {
                if let Expression::HashLiteral(HashLiteral { token: _, pairs }) = expression {
                    assert!(
                        pairs.len() == 3,
                        "hash.pairs has wrong length. got={}",
                        pairs.len()
                    );

                    let mut tests: HashMap<String, fn(&Expression)> = HashMap::new();
                    tests.insert(String::from("one"), |e| {
                        test_infix_expression(
                            e,
                            &ExpectedType::Ival(0),
                            "+",
                            &ExpectedType::Ival(1),
                        )
                    });
                    tests.insert(String::from("two"), |e| {
                        test_infix_expression(
                            e,
                            &ExpectedType::Ival(10),
                            "-",
                            &ExpectedType::Ival(8),
                        )
                    });
                    tests.insert(String::from("three"), |e| {
                        test_infix_expression(
                            e,
                            &ExpectedType::Ival(15),
                            "/",
                            &ExpectedType::Ival(5),
                        )
                    });

                    for (key, value) in pairs {
                        if let Expression::StringLiteral(literal) = key {
                            if let Some(test_func) = tests.get(&literal.string()) {
                                test_func(value);
                            } else {
                                panic!("No test function for key {:?} found", literal.string());
                            }
                        } else {
                            panic!("key is not StringLiteral. got={:?}", key);
                        }
                    }
                } else {
                    panic!("exp is not HashLiteral. got={:?}", expression);
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
thread 'parser::parser_test::test_parsing_empty_hash_literal' panicked at 'parser error: no prefix parse function for LBRACE found
no prefix parse function for RBRACE found', src/parser/parser_test.rs:40:5
note: run with `RUST_BACKTRACE=1` environment variable to display a backtrace.

thread 'parser::parser_test::test_parsing_hash_literals_string_keys' panicked at 'parser error: no prefix parse function for LBRACE found
no prefix parse function for COLON found
no prefix parse function for COMMA found
no prefix parse function for COLON found
no prefix parse function for COMMA found
no prefix parse function for COLON found
no prefix parse function for RBRACE found', src/parser/parser_test.rs:40:5

thread 'parser::parser_test::test_parsing_hash_literal_with_expressions' panicked at 'parser error: no prefix parse function for LBRACE found
no prefix parse function for COLON found
no prefix parse function for COMMA found
no prefix parse function for COLON found
no prefix parse function for COMMA found
no prefix parse function for COLON found
no prefix parse function for RBRACE found', src/parser/parser_test.rs:40:5
```

为大括号增加一个前缀解析函数：
```rust,noplaypen
// src/parser/parser.rs

impl Parser {
    pub fn new(l: Lexer) -> Parser {
// [...]
        p.register_prefix(TokenType::LBRACKET, |parser| parser.parse_array_literal());
        p.register_prefix(TokenType::LBRACE, |parser| parser.parse_hash_literal());
// [...]
    }

    fn parse_hash_literal(&mut self) -> Result<Expression, String> {
        let token = self.cur_token.clone();
        let mut pairs: HashMap<Expression, Expression> = HashMap::new();
        while !self.peek_token_is(&TokenType::RBRACE) {
            self.next_token();
            let key = self.parse_expression(Precedence::LOWEST)?;
            self.expect_peek(&TokenType::COLON)?;
            self.next_token();
            let value = self.parse_expression(Precedence::LOWEST)?;
            pairs.insert(key, value);

            if !self.peek_token_is(&TokenType::RBRACE) {
                self.expect_peek(&TokenType::COMMA)?;
            }
        }

        self.expect_peek(&TokenType::RBRACE)?;
        Ok(Expression::HashLiteral(HashLiteral {
            token: token,
            pairs: pairs,
        }))
    }
```
这里需要将Expression加入HashMap，这需要为Expression支持PartialEq、Eq和Hash trait，这里我们考虑的实现方式是比较对象字符串表示，以对象的字符串表示作为哈希值，代码如下：

```rust,noplaypen
// src/ast/ast.rs

use std::hash::Hash as StdHash;
use std::hash::Hasher;

impl Eq for Expression {}
impl PartialEq for Expression {
    fn eq(&self, other: &Self) -> bool {
        self.string() == other.string()
    }
}
impl StdHash for Expression {
    fn hash<H: Hasher>(&self, state: &mut H) {
        self.string().hash(state);
        state.finish();
    }
}
```
为了和后面我们自己定义的Hash区别开，这里将Rust的Hash trait重命名为StdHash。

测试通过！

## 哈希对象

扩展完词法分析器和解析器，下面我们扩展对象系统。本文中的实现方式跟原著差距较大，这是由Rust语言和Go语言的不同特性导致的。

直观感觉上任何Object都可以作为Hash的键，但实际上，我们的对象系统中真正有实用意义的键只包含三种：整数，布尔值和字符串。其它Object类型暂时不支持。

先增加测试用例：
```rust,noplaypen
// src/object/mod.rs

#[cfg(test)]
mod object_test;
```

```rust,noplaypen
// src/object/object_test.rs

use crate::object::*;

#[test]
fn test_string_hash_key() {
    let hello1 = StringObj {
        value: String::from("Hello World"),
    };
    let hello2 = StringObj {
        value: String::from("Hello World"),
    };
    let diff1 = StringObj {
        value: String::from("My name is johnny"),
    };
    let diff2 = StringObj {
        value: String::from("My name is johnny"),
    };

    assert!(
        hello1.hash_key() == hello2.hash_key(),
        "strings with same content have different hash keys"
    );
    assert!(
        diff1.hash_key() == diff2.hash_key(),
        "strings with same content have different hash keys"
    );
    assert!(
        hello1.hash_key() != diff1.hash_key(),
        "strings with different content have same hash keys"
    );
}
```

这里我们将能够作为哈希键的Object类型放到一起，定义一个HashKey枚举，定义如下：
```rust,noplaypen
// src/object/object.rs

use std::collections::hash_map::*;
use std::hash::Hash as StdHash;
use std::hash::Hasher;
use std::fmt::*;

#[derive(PartialEq, Eq, StdHash, Clone)]
pub enum HashKey {
    Integer(Integer),
    Boolean(Boolean),
    StringObj(StringObj),
}
impl HashKey {
    fn inspect(&self) -> String {
        match self {
            HashKey::Integer(i) => i.inspect(),
            HashKey::Boolean(b) => b.inspect(),
            HashKey::StringObj(s) => s.inspect(),
        }
    }
}

impl StdHash for Integer {
    fn hash<H: Hasher>(&self, state: &mut H) {
        self.get_type().hash(state);
        state.write_i64(self.value);
        state.finish();
    }
}
impl StdHash for Boolean {
    fn hash<H: Hasher>(&self, state: &mut H) {
        self.get_type().hash(state);
        if self.value {
            state.write_u8(1);
        } else {
            state.write_u8(0);
        }
        state.finish();
    }
}
impl StdHash for StringObj {
    fn hash<H: Hasher>(&self, state: &mut H) {
        self.get_type().hash(state);
        self.value.hash(state);
        state.finish();
    }
}
```
这里我们分别给出了整数、布尔值和字符串对象的哈希计算方法：类型名称和值都参与哈希计算。

下面为对象系统添加Hash对象定义：
```rust,noplaypen
// src/object/object.rs

#[derive(Clone)]
pub struct Hash {
    pub pairs: HashMap<HashKey, Object>,
}
impl Debug for Hash {
    fn fmt(&self, f: &mut Formatter<'_>) -> Result {
        write!(f, "Hash")
    }
}
impl Eq for Hash {}
impl PartialEq for Hash {
    fn eq(&self, _other: &Self) -> bool {
        false
    }
}
impl ObjectTrait for Hash {
    fn get_type(&self) -> &str {
        "HASH"
    }
    fn inspect(&self) -> String {
        format!(
            "{{{}}}",
            self.pairs
                .iter()
                .map(|(key, value)| { format!("{}: {}", key.inspect(), value.inspect()) })
                .collect::<Vec<String>>()
                .join(", ")
        )
    }
}

pub enum Object {
// [...]
    Hash(Hash),
}
impl ObjectTrait for Object {
    fn get_type(&self) -> String {
        match self {
// [...]
            Object::Hash(h) => h.get_type(),
        }
    }
    fn inspect(&self) -> String {
        match self {
// [...]
            Object::Hash(h) => h.inspect(),
        }
    }
}
```
其实就是将Rush HashMap封装一下。

## 哈希字面量求值

修改完词法分析器和解析器，在对象系统中加入Hash对象后，我们可以开始修改求值器了。

测试用例：
```rust,noplaypen
// src/evaluator/evaluator_test.rs

use std::collections::*;

#[test]
fn test_hash_literals() {
    let input = r#"let two = "two";
    {
        "one": 10 - 9,
        two: 1 + 1,
        "thr" + "ee": 6 / 2,
        4: 4,
        true: 5,
        false: 6
    }"#;

    let evaluated = test_eval(input);
    if let Object::Hash(Hash { pairs }) = evaluated {
        let mut expected: HashMap<HashKey, i64> = HashMap::new();
        expected.insert(
            StringObj {
                value: String::from("one"),
            }
            .hash_key(),
            1,
        );
        expected.insert(
            StringObj {
                value: String::from("two"),
            }
            .hash_key(),
            2,
        );
        expected.insert(
            StringObj {
                value: String::from("three"),
            }
            .hash_key(),
            3,
        );
        expected.insert(Integer { value: 4 }.hash_key(), 4);
        expected.insert(TRUE.hash_key(), 5);
        expected.insert(FALSE.hash_key(), 6);

        assert!(
            pairs.len() == expected.len(),
            "Hash has wrong num of pairs. got={}",
            pairs.len()
        );
        for (expected_key, expected_value) in expected.iter() {
            if let Some(pair) = pairs.get(expected_key) {
                test_integer_object(pair.clone(), *expected_value);
            } else {
                panic!("no pair for given key in pairs");
            }
        }
    } else {
        panic!("eval didn't return Hash. got={:?}", evaluated);
    }
}
```

前面提到过，不是所有的Object都能作为HashKey的，需要设计一种方式转换Object对象，某些能用来做HashKey的对象才可以用来求值，这里我设计了两个Rust trait：Hashable和AsHashable，用来转换Object为Hashable。

定义如下：
```rust,noplaypen
// src/object/object.rs

pub trait AsHashable {
    fn as_hashable(&self) -> Option<&dyn Hashable>;
}
impl AsHashable for Object {
    fn as_hashable(&self) -> Option<&dyn Hashable> {
        match self {
            Object::Integer(i) => Some(i),
            Object::Boolean(b) => Some(b),
            Object::StringObj(s) => Some(s),
            _ => None,
        }
    }
}

pub trait Hashable {
    fn hash_key(&self) -> HashKey;
}
impl Hashable for Boolean {
    fn hash_key(&self) -> HashKey {
        HashKey::Boolean(self.clone())
    }
}
impl Hashable for Integer {
    fn hash_key(&self) -> HashKey {
        HashKey::Integer(self.clone())
    }
}
impl Hashable for StringObj {
    fn hash_key(&self) -> HashKey {
        HashKey::StringObj(self.clone())
    }
}
```
仅当对象为整数、布尔值和字符串时，可以将其转换成Hashable，然后通过hash_key方法取得HashKey对象。

求值器中修改如下：
```rust,noplaypen
// src/evaluator/evaluator.rs

pub fn eval(node: Node, env: Rc<RefCell<Environment>>) -> Result<Object, String> {
    match node {
// [...]
        Node::Expression(Expression::HashLiteral(hash_literal)) => {
            eval_hash_literal(hash_literal, Rc::clone(&env))
        }
// [...]        
    }
}
```
其中eval_hash_literal为：
```rust,noplaypen
// src/evaluator/evaluator.rs

use std::collections::*;

fn eval_hash_literal(node: HashLiteral, env: Rc<RefCell<Environment>>) -> Result<Object, String> {
    let mut pairs: HashMap<HashKey, Object> = HashMap::new();

    for (key_node, value_node) in node.pairs.iter() {
        let key = eval(Node::Expression(key_node.clone()), Rc::clone(&env))?;
        if let Some(hash_key) = key.as_hashable() {
            let value = eval(Node::Expression(value_node.clone()), Rc::clone(&env))?;
            let hashed = hash_key.hash_key();
            pairs.insert(hashed, value);
        } else {
            panic!("unusable as hash key: {}", key.get_type());
        }
    }
    Ok(Object::Hash(Hash { pairs: pairs }))
}
```
测试通过！

运行cargo run启动REPL，输入如下：
```
Hello, This is the Monkey programming language!
Feel free to type in commands
>> {"name": "Monkey", "age": 0, "type": "Language", "status": "awesome"}
{age: 0, name: Monkey, type: Language, status: awesome}
>> 
```
不错！但是我们还不能访问哈希中的元素，例如：
```
Hello, This is the Monkey programming language!
Feel free to type in commands
>> let bob = {"name": "Bob", "age": 99};
{name: Bob, age: 99}
>> bob["name"]
ERROR: index operator not supported: HASH
>> 
```
我马上修！

## 哈希索引表达式求值

增加测试用例：
```rust,noplaypen
// src/evaluator/evaluator_test.rs

#[test]
fn test_hash_index_expressions() {
    let tests = vec![
        (
            r#"{"foo": 5}["foo"]"#,
            Object::Integer(Integer { value: 5 }),
        ),
        (r#"{"foo": 5}["bar"]"#, Object::Null(NULL)),
        (
            r#"let key = "foo"; {"foo": 5}[key]"#,
            Object::Integer(Integer { value: 5 }),
        ),
        (r#"{}["foo"]"#, Object::Null(NULL)),
        ("{5: 5} [5]", Object::Integer(Integer { value: 5 })),
        ("{true: 5}[true]", Object::Integer(Integer { value: 5 })),
        ("{false: 5}[false]", Object::Integer(Integer { value: 5 })),
    ];

    for tt in tests.iter() {
        let evaluated = test_eval(tt.0);
        if let Object::Integer(integer) = &tt.1 {
            test_integer_object(evaluated, integer.value);
        } else {
            test_null_object(evaluated);
        }
    }
}

fn test_error_handling() {
    let tests = [
// [...]
        (
            r#"{"name": "Monkey"}[fn(x){ x }];"#,
            "unusable as hash key: FUNCTION",
        ),
    ];
```
测试失败结果如下：
```
thread 'evaluator::tests::test_hash_index_expressions' panicked at 'object is not Integer. got=ErrorObj(ErrorObj { message: "index operator not supported: HASH" })', src/evaluator/evaluator_test.rs:661:13
thread 'evaluator::tests::test_error_handling' panicked at 'wrong error message. expected=unusable as hash key: FUNCTION, got=index operator not supported: HASH', src/evaluator/evaluator_test.rs:827:17
```

扩展索引表达式求值函数：
```rust,noplaypen
// src/evaluator/evaluator.rs

fn eval_index_expression(left: Object, index: Object) -> Result<Object, String> {
    if let Object::Array(Array { elements }) = &left {
        if let Object::Integer(Integer { value }) = index {
            return eval_array_index_expression(elements, value);
        }
    } else if let Object::Hash(hash_obj) = &left {
        return eval_hash_index_expression(hash_obj, index);
    }
    Err(format!("index operator not supported: {}", left.get_type()))
}
```
增加了哈希索引表达式求值方法。具体定义如下：

```rust,noplaypen
// src/evaluator/evaluator.rs

fn eval_hash_index_expression(hash: &Hash, index: Object) -> Result<Object, String> {
    if let Some(key) = index.as_hashable() {
        if let Some(pair) = hash.pairs.get(&key.hash_key()) {
            return Ok(pair.clone());
        }
        Ok(Object::Null(NULL))
    } else {
        Err(format!("unusable as hash key: {}", index.get_type()))
    }
}
```
测试通过！

执行cargo run：
```
Hello, This is the Monkey programming language!
Feel free to type in commands
>> let people = [{"name": "Alice", "age": 24}, {"name": "Anna", "age": 28}];
[{age: 24, name: Alice}, {name: Anna, age: 28}]
>> people[0]["name"];
Alice
>> people[1]["age"];
28
>> people[1]["age"] + people[0]["age"];
52
>> let getName = fn(person) { person["name"]; };
fn(person) {
(person[name])
}
>> getName(people[0]);
Alice
>> getName(people[1]);
Anna
>> 
```