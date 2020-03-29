# 树遍历解释器

我们要构建的是一个树遍历解释器。我们将使用解析器为我们构建AST并“即时”解释它，而无需任何预处理或编译步骤。

实际上，我们只需要两件事：一个遍历树的求值器，以及一种以宿主语言Rust表示Monkey值的方法。求值器听起来高大上，但只是一个名为eval的函数。它的工作是求值AST。下面的伪代码说明了在解释上下文中“即时求值”和“树遍历”的含义：

```js
function eval(astNode) {
    if (astNode is integerliteral) {
        return astNode.integerValue
    } else if (astNode is booleanLiteral) {
        return astNode.booleanValue
    } else if (astNode is infixExpression) {
        leftEvaluated = eval(astNode.Left) 
        rightEvaluated = eval(astNode.Right)
        if astNode.Operator == "+" {
            return leftEvaluated + rightEvaluated
        } else if ast.Operator == "-" {
            return leftEvaluated - rightEvaluated
        } 
    }
}
```

这里的eval也是递归执行的。例如当遇到infixExpression时，递归调用求值左子树和右子树。

但是返回值又是什么样子的呢？