# 词法分析

解释一门语言的时候需要把源代码转换成其它形式，如下：

![f1-1](image/f1-1.png)

第一个转换，从源代码到Token（译为记号，本文为了简便，直接使用Token），被称作“词法分析”，我们需要实现一个词法分析器（Lexer）。

第二个转换，从Token到抽象语法树（Abstract Syntax Tree，简称AST），被称作“解析”，我们需要实现一个解析器（Parser）。

例如，输入到词法分析器的源代码为：
```js
let x = 5 + 5;
```
词法分析器的输出为：
```js
[
    LET,
    IDENTIFIER("x"),
    EQUAL_SIGN,
    INTEGER(5),
    PLUS_SIGN,
    INTEGER(5),
    SEMICOLON,
]
```
这里空格被忽略了。