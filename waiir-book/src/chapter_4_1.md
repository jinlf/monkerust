# 给符号赋予含义

求值的过程使源代码具有了真正的含义。

例如：
```js
let num = 5; 
if (num) {
    return a; 
} else {
    return b; 
}
```
中表达式返回a还是b是由解释器决定的，需要确定整数5是真值还是假值，有些语言中返回真值，还有一些语言不支持这种写法。

再例如：
```js
let one = fn() { 
    printLine("one"); 
    return 1;
};
let two = fn() { 
    printLine("two"); 
    return 2;
};
add(one(), two());
```
是先输出one还是先输出two也是由解释器决定的，是根据参数求值顺序来决定。

本章将会遇到许多这种小的选择。