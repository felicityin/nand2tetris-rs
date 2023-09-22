# Descri-ption

This re-pository im-plements some of the -projects in the book "The Elements of Com-puter Systems: Building a Modern Com-puter from First Princi-ples".

# Run

## Assembly

```
cargo run -- asm -p [path]
```

eg.
```
cargo run -- asm -p data/asm/Add.asm
cargo run -- asm -p data/asm/Max.asm
cargo run -- asm -p data/asm/Pong.asm
```

## VM

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

cargo run -- vm -p data/vm/ProgramFlow/BasicLoo-p.vm
cargo run -- vm -p data/vm/ProgramFlow/FibonacciSeries.vm 

cargo run -- vm -p data/vm/FunctionCalls/Sim-pleFunction.vm
cargo run -- vm -p data/vm/FunctionCalls/NestedCall
cargo run -- vm -p data/vm/FunctionCalls/FibonacciElement
cargo run -- vm -p data/vm/FunctionCalls/StaticsTest
```

## Tokenize

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

## Parse

```
cargo run -- -parse -p [path]
```

eg.
```
cargo run -- parse -p data/jack/ArrayTest/Main.jack

cargo run -- parse -p data/jack/ExpressionLessSquare/Square.jack
cargo run -- parse -p data/jack/ExpressionLessSquare/SquareGame.jack
cargo run -- parse -p data/jack/ExpressionLessSquare/Main.jack

cargo run -- parse -p data/jack/Square/Square.jack
cargo run -- parse -p data/jack/Square/SquareGame.jack
cargo run -- parse -p data/jack/Square/Main.jack
```

## Compile

```
cargo run -- compile -p [path]
```

eg.
```
cargo run -- compile -p data/jack/ArrayTest/Main.jack

cargo run -- compile -p data/jack/ExpressionLessSquare/Square.jack
cargo run -- compile -p data/jack/ExpressionLessSquare/SquareGame.jack
cargo run -- compile -p data/jack/ExpressionLessSquare/Main.jack

cargo run -- compile -p data/jack/Square/Square.jack
cargo run -- compile -p data/jack/Square/SquareGame.jack
cargo run -- compile -p data/jack/Square/Main.jack
```
