# Description

This repository implements some of the projects in the book "The Elements of Computer Systems: Building a Modern Computer from First Principles".

# Assembly

```
cargo run -- asm -p [path]
```

eg.
```
cargo run -- asm -p data/asm/Add.asm
cargo run -- asm -p data/asm/Max.asm
cargo run -- asm -p data/asm/Pong.asm
```

# VM

```
cargo run -- vm -p [path]
```

eg.
```
cargo run -- vm -p data/vm/StackArithmetic/SimpleAdd.vm
cargo run -- vm -p data/vm/StackArithmetic/StaticTest.vm

cargo run -- vm -p data/vm/MemoryAccess/BasicTest.vm
cargo run -- vm -p data/vm/MemoryAccess/PointerTest.vm 
cargo run -- vm -p data/vm/MemoryAccess/StaticTest.vm

cargo run -- vm -p data/vm/ProgramFlow/BasicLoop.vm
cargo run -- vm -p data/vm/ProgramFlow/FibonacciSeries.vm 

cargo run -- vm -p data/vm/FunctionCalls/SimpleFunction.vm
cargo run -- vm -p data/vm/FunctionCalls/NestedCall
cargo run -- vm -p data/vm/FunctionCalls/FibonacciElement
cargo run -- vm -p data/vm/FunctionCalls/StaticsTest
```

# Tokenize

```
cargo run -- token -p [path]
```

eg.
```
cargo run -- token -p data/jack/ArrayTest/Main.jack

cargo run -- token -p data/jack/ExpressionLessSquare/Square.jack
cargo run -- token -p data/jack/ExpressionLessSquare/SquareGame.jack
cargo run -- token -p data/jack/ExpressionLessSquare/Main.jack

cargo run -- token -p data/jack/Square/Square.jack
cargo run -- token -p data/jack/Square/SquareGame.jack
cargo run -- token -p data/jack/Square/Main.jack
```
