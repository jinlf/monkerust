# Monkey编程语言与解释器

![Monkey语言官方Logo](image/monkey.png "Monkey语言官方Logo")

[Monkey语言](https://monkeylang.org)是原书作者Thorsten Ball设计的一门编程语言，简约而不简单，特点如下：
- 类似于C的语法
- 变量绑定
- 整数和布尔值
- 算术表达式
- 内置函数
- 一等函数和高阶函数
- 闭包
- 字符串数据结构
- 数组数据结构
- 哈希数据结构

变量绑定的方法如下：
```js
let age = 1;
let name = "Monkey";
let result = 10 * (20 / 2);
```
除了整数，布尔值和字符串之外，Monkey语言还支持数组和哈希，对应的变量绑定方法如下：
```js
let myArray = [1, 2, 3, 4, 5];
let thorsten = {"name": "Thorsten", "age": 28};
```
访问其内容的方式：
```js
myArray[0] // => 1 
thorsten["name"] // => "Thorsten"
```
使用let语句还可以绑定函数，如下：
```js
let add = fn(a, b) { return a + b; };
```
支持使用return语句显式地从函数返回，也支持不使用return语句的隐式的返回方式：
```js
let add = fn(a, b) { a + b; };
```
例如，计算斐波那契数列的（递归）函数如下：
```js
let fibonacci = fn(x) { 
    if (x == 0) {
        0
    } else {
        if (x == 1) {
            1
        } else {
            fibonacci(x - 1) + fibonacci(x - 2);
        } 
    }
};
```
Monkey语言支持高阶函数：
```js
let twice = fn(f, x) { 
    return f(f(x));
};
let addTwo = fn(x) { 
    return x + 2;
};
twice(addTwo, 2); // => 6
```
这里调用twice函数使用了两个实参，一个是函数addTwo，一个是整数2。执行时调用了addTwo两次，第一次使用2作为参数，第二次使用第一次调用的返回值做为参数，最终返回6。

在Monkey中，函数与整数、字符串一样，都是值。这种特性被称为“first class functions”（一等函数，即函数是Monkey语言的一等公民）。

本文实现的解释器有以下几个部分：
- 词法分析器
- 解析器
- 抽象语法树
- 内部对象系统
- 求值器

