# Monkey编程语言与解释器

![Monkey语言官方Logo](image/monkey.png "Monkey语言官方Logo")

[Monkey语言](https://monkeylang.org)是原书作者Thorsten Ball设计的一门适用于教学的简单的编程语言，特点如下：
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

绑定变量的方法如下
```js
let age = 1;
let name = "Monkey";
let result = 10 * (20 / 2);
```
除了整数，布尔值和字符串之外，数组和哈希的绑定示例如下：
```js
let myArray = [1, 2, 3, 4, 5];
let thorsten = {"name": "Thorsten", "age": 28};
```
访问方法如下：
```js
myArray[0] // => 1 
thorsten["name"] // => "Thorsten"
```
赋值let语句可以绑定函数，如下：
```js
let add = fn(a, b) { return a + b; };
```
函数中可以使用return语句显式返回，也可以不用return，隐式返回，例如：
```js
let add = fn(a, b) { a + b; };
```
计算斐波那契数列的（递归）函数如下：
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
Monkey语言支持高阶函数，如下：
```js
let twice = fn(f, x) { 
    return f(f(x));
};
let addTwo = fn(x) { 
    return x + 2;
};
twice(addTwo, 2); // => 6
```
这里调用twice函数使用了两个实参，一个是函数addTwo，一个是整数2。执行时调用了addTwo两次，第一次用2作为参数，第二次用第一次调用的返回值做为参数，最终返回6。

在Monkey中，函数与整型、字符串一样，都是值。这种特性被称为“first class functions”（一等函数，即函数是Monkey语言的一等公民）。

本文实现的解释器有以下几个部分：
- 词法分析器
- 解析器
- 抽象语法树
- 内部对象系统
- 求值器

