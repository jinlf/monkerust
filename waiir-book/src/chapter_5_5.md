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
上面例子中mhHash有三个键，都是字符串。我们可以用索引操作符表达式来访问值，只是用到的索引值是字符串。
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
这里有一个词法分析器不支持的符号“：”，先添加进Token类型中：

```rust,noplaypne
// src/token.rs

pub enum TokenType {
// [...]
    COLON,     // :
}
```

测试用例：
```rust,noplaypen
// src/lexer_test.rs

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
// src/lexer.rs

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
// src/ast.rs

use std::collections::*;

#[derive(Clone)]
pub struct HashLiteral {
    pub token: Token,
    pub pairs: HashMap<Expression, Expression>,
}
impl NodeTrait for HashLiteral {
    fn token_literal(&self) -> String {
        self.token.literal.clone()
    }
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
    fn token_literal(&self) -> String {
        match self {
// [...]
            Expression::HashLiteral(hash_literal) => hash_literal.token_literal(),
        }
    }
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
// src/parser_test.rs

use std::collections::*;

#[test]
fn test_parsing_hash_literals_string_keys() {
    let input = r#"{"one": 1, "two": 2, "three": 3}"#;
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
                        assert!(false, "key is not StringLiteral. got={:?}", key);
                    }
                }
            } else {
                assert!(false, "exp is not HashLiteral. got={:?}", expression);
            }
        } else {
            assert!(false, "parse error");
        }
    } else {
        assert!(false, "parse error");
    }
}
```
需要能够支持空哈希，测试用例如下：
```rust,noplaypen
// src/parser_test.rs

#[test]
fn test_parsing_empty_hash_literal() {
    let input = "{}";
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
            if let Expression::HashLiteral(HashLiteral { token: _, pairs }) = expression {
                assert!(
                    pairs.len() == 0,
                    "hash.pairs has wrong length. got={}",
                    pairs.len()
                );
            } else {
                assert!(false, "exp is not HashLiteral. got={:?}", expression);
            }
        } else {
            assert!(false, "parse error");
        }
    } else {
        assert!(false, "parse error");
    }
}
```
哈希键和值都可以是表达式，测试用例如下：
```rust,noplaypen
// src/parser_test.rs

#[test]
fn test_parsing_hash_literal_with_expressions() {
    let input = r#"{"one": 0 + 1, "two": 10 - 8, "three": 15 / 5}"#;
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
                        &*Box::new(0 as i64),
                        String::from("+"),
                        &*Box::new(1 as i64),
                    )
                });
                tests.insert(String::from("two"), |e| {
                    test_infix_expression(
                        e,
                        &*Box::new(10 as i64),
                        String::from("-"),
                        &*Box::new(8 as i64),
                    )
                });
                tests.insert(String::from("three"), |e| {
                    test_infix_expression(
                        e,
                        &*Box::new(15 as i64),
                        String::from("/"),
                        &*Box::new(5 as i64),
                    )
                });

                for (key, value) in pairs {
                    if let Expression::StringLiteral(literal) = key {
                        if let Some(test_func) = tests.get(&literal.string()) {
                            test_func(value);
                        } else {
                            assert!(
                                false,
                                "No test function for key {:?} found",
                                literal.string()
                            );
                        }
                    } else {
                        assert!(false, "key is not StringLiteral. got={:?}", key);
                    }
                }
            } else {
                assert!(false, "exp is not HashLiteral. got={:?}", expression);
            }
        } else {
            assert!(false, "parse error");
        }
    } else {
        assert!(false, "parse error");
    }
}
```
测试失败结果如下：
```
thread 'parser::tests::test_parsing_empty_hash_literal' panicked at 'parser has 2 errors
parser error: "no prefix parse function for LBRACE found"
parser error: "no prefix parse function for RBRACE found"
', src/parser_test.rs:629:9

thread 'parser::tests::test_parsing_hash_literal_with_expressions' panicked at 'parser has 7 errors
parser error: "no prefix parse function for LBRACE found"
parser error: "no prefix parse function for COLON found"
parser error: "no prefix parse function for COMMA found"
parser error: "no prefix parse function for COLON found"
parser error: "no prefix parse function for COMMA found"
parser error: "no prefix parse function for COLON found"
parser error: "no prefix parse function for RBRACE found"
', src/parser_test.rs:629:9

thread 'parser::tests::test_parsing_hash_literals_string_keys' panicked at 'parser has 7 errors
parser error: "no prefix parse function for LBRACE found"
parser error: "no prefix parse function for COLON found"
parser error: "no prefix parse function for COMMA found"
parser error: "no prefix parse function for COLON found"
parser error: "no prefix parse function for COMMA found"
parser error: "no prefix parse function for COLON found"
parser error: "no prefix parse function for RBRACE found"
', src/parser_test.rs:629:9
```

为大括号增加一个前缀解析函数：
```rust,noplaypen
// src/parser.rs

use std::collections::*;

    fn parse_expression(&mut self, precedence: Precedence) -> Option<Expression> {
        let mut left_exp: Option<Expression>;
        let tk_type = self.cur_token.tk_type.clone();
        match tk_type {
// [...]
            TokenType::LBRACE => left_exp = self.parse_hash_literal(),
// [...]
        }
// [...]
    }

    fn parse_hash_literal(&mut self) -> Option<Expression> {
        let token = self.cur_token.clone();
        let mut pairs: HashMap<Expression, Expression> = HashMap::new();
        while !self.peek_token_is(TokenType::RBRACE) {
            self.next_token();
            let key = self.parse_expression(Precedence::LOWEST);
            if key.is_none() {
                return None;
            }

            if !self.expect_peek(TokenType::COLON) {
                return None;
            }

            self.next_token();
            let value = self.parse_expression(Precedence::LOWEST);
            if value.is_none() {
                return None;
            }

            pairs.insert(key.unwrap(), value.unwrap());

            if !self.peek_token_is(TokenType::RBRACE) && !self.expect_peek(TokenType::COMMA) {
                return None;
            }
        }

        if !self.expect_peek(TokenType::RBRACE) {
            return None;
        }

        Some(Expression::HashLiteral(HashLiteral {
            token: token,
            pairs: pairs,
        }))
    }
```
这里需要将Expression加入HashMap，这需要为Expression支持PartialEq、Eq和Hash trait，这里我们考虑的实现方式是比较对象的地址，并以对象的地址作为哈希值，代码如下：

```rust,noplaypen
// src/ast.rs

use std::hash::Hash as StdHash;
use std::hash::Hasher;

impl Eq for Expression {}
impl PartialEq for Expression {
    fn eq(&self, other: &Self) -> bool {
        let addr = self as *const Expression as usize;
        let other_addr = other as *const Expression as usize;
        addr == other_addr
    }
}
impl StdHash for Expression {
    fn hash<H: Hasher>(&self, state: &mut H) {
        let addr = self as *const Expression as usize;
        state.write_usize(addr);
        state.finish();
    }
}
```
为了和后面我们自己定义的Hash区别开，这里将Rust的Hash trait重命名为StdHash。

测试通过！

## 哈希对象

扩展完词法分析器和解析器，下面我们扩展对象系统。

```rust,noplaypen
// src/object_test.rs

use super::object::*;

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

lib.rs中加入
```rust,noplaypen
// src/lib.rs

#[cfg(test)]
mod object_test;
```

```rust,noplaypen
// src/object.rs

use std::collections::hash_map::*;
use std::hash::Hash;
use std::hash::Hasher;
use std::fmt::*;

#[derive(Clone, PartialEq, Eq, StdHash)]
pub struct HashKey {
    pub obj_type: String,
    pub value: u64,
}
pub trait Hashable {
    fn hash_key(&self) -> HashKey;
}
impl Hashable for Boolean {
    fn hash_key(&self) -> HashKey {
        let mut value: u64 = 0;
        if self.value {
            value = 1;
        }
        HashKey {
            obj_type: self.get_type(),
            value: value,
        }
    }
}
impl Hashable for Integer {
    fn hash_key(&self) -> HashKey {
        HashKey {
            obj_type: self.get_type(),
            value: self.value as u64,
        }
    }
}
impl Hashable for StringObj {
    fn hash_key(&self) -> HashKey {
        let mut h = DefaultHasher::new();
        self.value.hash(&mut h);
        HashKey {
            obj_type: self.get_type(),
            value: h.finish(),
        }
    }
}
```

```rust,noplaypen
// src/object.rs

#[derive(Clone)]
pub struct HashPair {
    pub key: Object,
    pub value: Object,
}
#[derive(Clone)]
pub struct Hash {
    pub pairs: HashMap<HashKey, HashPair>,
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
    fn get_type(&self) -> String {
        String::from("HASH")
    }
    fn inspect(&self) -> String {
        format!(
            "{{{}}}",
            self.pairs
                .iter()
                .map(|(_, pair)| { format!("{}: {}", pair.key.inspect(), pair.value.inspect()) })
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

## 哈希表字面量求值

```rust,noplaypen
// src/evaluator_test.rs

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
    if let Some(Object::Hash(Hash { pairs })) = evaluated {
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
                test_integer_object(Some(pair.value.clone()), *expected_value);
            } else {
                assert!(false, "no pair for given key in pairs");
            }
        }
    } else {
        assert!(false, "eval didn't return Hash. got={:?}", evaluated);
    }
}
```
测试输出
```
thread 'evaluator_test::test_hash_literals' panicked at 'eval didn't return Hash. got=None', src/evaluator_test.rs:483:9
```

```rust,noplaypen
// src/object.rs

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
```


```rust,noplaypen
// src/evaluator.rs

pub fn eval(node: Node, env: Rc<RefCell<Environment>>) -> Option<Object> {
    match node {
// [...]
        Node::Expression(Expression::HashLiteral(hash_literal)) => {
            eval_hash_literal(hash_literal, Rc::clone(&env))
        }
    }
}
```

```rust,noplaypen
// src/evaluator.rs

use std::collections::*;

fn eval_hash_literal(node: HashLiteral, env: Rc<RefCell<Environment>>) -> Option<Object> {
    let mut pairs: HashMap<HashKey, HashPair> = HashMap::new();

    for (key_node, value_node) in node.pairs.iter() {
        let key = eval(Node::Expression(key_node.clone()), Rc::clone(&env));
        if is_error(&key) {
            return key;
        }
        if key.is_none() {
            return None;
        }

        if let Some(hash_key) = key.as_ref().unwrap().as_hashable() {
            let value = eval(Node::Expression(value_node.clone()), Rc::clone(&env));
            if is_error(&value) {
                return value;
            }
            if value.is_none() {
                return None;
            }
            let hashed = hash_key.hash_key();
            pairs.insert(
                hashed,
                HashPair {
                    key: key.unwrap(),
                    value: value.unwrap(),
                },
            );
        } else {
            assert!(false, "unusable as hash key: {}", get_type(&key));
        }
    }
    Some(Object::Hash(Hash { pairs: pairs }))
}
```
测试通过！

## 带哈希的索引表达式求值

```rust,noplaypen
// src/evaluator_test.rs

#[test]
fn test_hash_index_expressions() {
    let tests: [(&str, Option<Object>); 7] = [
        (
            r#"{"foo": 5}["foo"]"#,
            Some(Object::Integer(Integer { value: 5 })),
        ),
        (r#"{"foo": 5}["bar"]"#, Some(Object::Null(NULL))),
        (
            r#"let key = "foo"; {"foo": 5}[key]"#,
            Some(Object::Integer(Integer { value: 5 })),
        ),
        (r#"{}["foo"]"#, Some(Object::Null(NULL))),
        ("{5: 5} [5]", Some(Object::Integer(Integer { value: 5 }))),
        (
            "{true: 5}[true]",
            Some(Object::Integer(Integer { value: 5 })),
        ),
        (
            "{false: 5}[false]",
            Some(Object::Integer(Integer { value: 5 })),
        ),
    ];

    for tt in tests.iter() {
        let evaluated = test_eval(tt.0);
        if let Some(Object::Integer(integer)) = &tt.1 {
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
测试结果
```
thread 'evaluator::tests::test_hash_index_expressions' panicked at 'object is not Integer. got=Some(ErrorObj(ErrorObj { message: "index operator not supported: HASH" }))', src/evaluator_test.rs:661:13
thread 'evaluator::tests::test_error_handling' panicked at 'wrong error message. expected=unusable as hash key: FUNCTION, got=index operator not supported: HASH', src/evaluator_test.rs:827:17
```

```rust,noplaypen
// src/evaluator.rs

fn eval_index_expression(left: Object, index: Object) -> Option<Object> {
    if let Object::Array(_) = left {
        if let Object::Integer(_) = index {
            return eval_array_index_expression(left, index);
        }
    } else if let Object::Hash(hash_obj) = left {
        return eval_hash_index_expression(hash_obj, index);
    }
    new_error(format!("index operator not supported: {}", left.get_type()))
}
```

```rust,noplaypen
// src/evaluator.rs

fn eval_hash_index_expression(hash: Hash, index: Object) -> Option<Object> {
    if let Some(key) = index.as_hashable() {
        if let Some(pair) = hash.pairs.get(&key.hash_key()) {
            return Some(pair.value.clone());
        }
        return Some(Object::Null(NULL));
    } else {
        return new_error(format!("unusable as hash key: {}", index.get_type()));
    }
}
```

用cargo run执行
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