# 如何使用本书

本文结构和内容参考《Writing An Interpreter in Go》([https://interpreterbook.com](https://interpreterbook.com))，不追求写出比原版更高效、更稳定的代码，仅考虑如何用Rust来实现相同的功能。

很早就有人用Rust语言实现了Monkey语言的解释器:
- [https://github.com/shioyama18/monkey-wasm](https://github.com/shioyama18/monkey-wasm)；
- [https://github.com/wadackel/rs-monkey-lang](https://github.com/wadackel/rs-monkey-lang)；
- [https://github.com/Rydgel/monkey-rust](https://github.com/Rydgel/monkey-rust)；
- [https://github.com/utatti/monkey-rs](https://github.com/utatti/monkey-rs)；
- [https://github.com/JonnyWalker81/monkey_interpreter](https://github.com/JonnyWalker81/monkey_interpreter)；
- [https://github.com/rainliu/monkey](https://github.com/rainliu/monkey)。

本文没有参考这些实现（以后也许会考察一下），仅考虑最大限度地符合原著。

本文不包含Rust安装、调试和入门语法等内容，也不包含编译原理的深入介绍，只是展示用Rust编写一门语言解释器的过程。