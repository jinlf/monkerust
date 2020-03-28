# 如何使用本书

本书结构和内容参考[《Writing An Interpreter in Go》](https://interpreterbook.com)，不追求写出比原版更高效、更稳定的代码，仅考虑相同的功能如何用Rust来实现。

很早就有人用Rust语言实现过了Monkey语言的解释器，感兴趣的请参考[链接](https://github.com/tsuyoshiwada/rs-monkey-lang)。本书没有参考这个实现（以后也许会考察一下），仅考虑最大限度地符合原著。

本文不包含Rust安装、调试、入门语法等内容，也不包含编译原理的深入介绍，只是展示用Rust编写一门语言解释器的过程。