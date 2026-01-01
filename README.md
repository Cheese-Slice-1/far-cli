# far-cli
======

FAR (Float ARray) - the only language where every value is a floating point number array

## Basic concepts
- There's a buffer and a stack.
    - The buffer stores temporary values and is overwritten every time a new value is loaded in it. You can push the buffer into the stack.
    - The stack stores values in a LIFO (Last In First Out) order. You can push, pop and duplicate values.
- Any sequence of numbers ("1 2 3", "10 77 2.8", etc.) is an array that gets put in the buffer.
- Alphabetic characters that can
- All instructions that give a result put it in the buffer, NEVER into the stack.
- Instructions can't be used in an affix-like manner. They must be separated with spaces from numbers and other instructions.
    - Except from - and ., which are used for number parsing, and #, that is used as "#(id)" to denote interactive-interpreter-specific commands (like quiting the interpreter, printing the contents of the buffer and the stack, reseting them, etc.).

## Examples
Hello world:
```cpp
72 101 108 108 111 44 32 119 111 114 108 100 33 $
```

\*Imperfect cat:
```cpp
, $
```
###### \*for the time being (iirc), `,` does NOT keep spaces


