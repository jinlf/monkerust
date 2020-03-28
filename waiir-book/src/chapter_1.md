# 介绍

我最近读了一本书[《Writing An Interpreter in Go》](https://interpreterbook.com)，书中用Go语言编写了一个自定义编程语言Monkey语言的解释器。我觉得很有意思。恰逢我正在学习Rust语言，需要一个练手的项目，于是决定用Rust语言重写一遍这个解释器。

本书结构和内容参考《Writing An Interpreter in Go》，不追求写出比原版更高效、更稳定的代码，仅考虑相同的功能如何用Rust来实现。

很早就有人实现过了Monkey语言的Rust解释器，感兴趣的请参考[链接](https://github.com/tsuyoshiwada/rs-monkey-lang)。本书没有参考这些实现，仅考虑最大限度地符合原著。

[Rust语言](https://www.rust-lang.org)十分强大，本人也是刚刚接触，没有经验。实现解释器的过程使用的Rust语法非常有限，仅能作为入门级练手项目。进一步学习Rust语言还需要更多的项目练习。

解释器是属于编译原理领域的一类产品，本书实现的Monkey语言解释器能够实现原书中的所有功能。

![Monkey语言官方Logo](image/logo.png "Monkey语言官方Logo")

[Monkey语言](https://monkeylang.org)是原书作者设计的一门非常适用于教学的简单的脚本语言，具体功能请参考原作。

下面创建一个Rust项目
```
$ cargo new waiir
$ cd waiir
$ cargo run
```
输出
```
Hello, world!
```
创建成功！