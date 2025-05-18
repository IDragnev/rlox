# An interpreter for the Lox programming language.
Lox is a dynamically typed language with syntax very close to that of C.

## Usage
```
rlox [path-to-script-file]
```

You can use the interpreter in [REPL](https://en.wikipedia.org/wiki/Read%E2%80%93eval%E2%80%93print_loop) mode or by running a script file.  
- Running the interpreter with no argument loads it in REPL mode. To exit the REPL type *q*.
- Running the interpreter with a path to a script loads the script and tries to execute it.

## Types
- **bool** - values can be *true* and *false*
- **number** - all numbers are represented as double-precision floating-point numbers. They must not have a trailing dot (*1.* is not allowed as a literal, while *1.0* and *1* are ok).
- **string** - string literals are sequences of characters enclosed in double quotes, such as *"hello"*.
- **functions** - Lox has [first-class functions](https://en.wikipedia.org/wiki/First-class_function). It supports passing functions as arguments to other functions, returning them as the values from other functions, and assigning them to variables.
- **nil** - the **nil** type has a single value - *nil*. It represents the [null value](https://en.wikipedia.org/wiki/Nullable_type). It is the value of any uninitialized variable and the default return value of functions.
- **classes** - user defined types with methods and dynamic fields. Inheritance is coming soon.

## Working with values
### Variables and functions
Variables are declared with the *var* keyword. Variable names must start with a letter or an underscore, can contain letters, digits, and underscores, and cannot include spaces or special characters. Additionally, they must not be reserved keywords and are case-sensitive:
```
var a = 10;
var A = a; // A = 10;
var b; // b = nil
var c = "hello";
var d = false;
```
Functions are declared with the *fun* keyword:
```
fun sum(a, b) { return a + b; }
fun empty() { } // returns nil
```
Functions can be local and can be treated as any other value:
```
fun plus1() {
    fun local(a) {
        return a + 1;
    }

    return local;
}
var f = plus1();
var a = f(2); // a = 3;
```
You can inspect values by printing them:
```
var a = 10;
fun f() {}
print a; // 10
print f; // <fun f>

```

### Truthiness
**nil** and **false** are falsey, everything else is truthy:
```
if (0) { print "true"; } // prints "true"
if (nil) { print "oh no"; } else { print "phew"; } // prints "phew"
```

### Control flow
#### if
The else branch is optional but you can only have one else branch (no else if).
```
// else is optional
if (a) {
    print "yes";
}

// if and else
if (a) {
    print "yes";
}
else {
    print "no";
}
```

#### Relational operators
Relational operators are used to compare two values and return true or false depending on the comparison:

- == equal to
- != not equal to
- \> greater than
- < less than
- \>= greater than or equal to
- <= less than or equal to


#### Loops
Lox has *while* and *for* loops. Their syntax is like in C.
```
var a = 10;
while (a > 0) {
    print a;
    a = a - 1;
}

var a = 10;
while (true) {
    if (a <= 0) {
        break;
    }
    print a;
    a = a - 1;
}

for (var a = 10; a > 0; a = a - 1) {
    print a;
}
```

#### Logical operators
Lox uses *!* for negation and the keywords *and* and *or* for the corresponding logical operators. They short circuit:
```
print !true; // false

var a = false or "true" or 1 or 2;
print a; // "true";

var b = true and 1 and nil and 1;
print b; // nil
```

#### Classes and instances
* Classes are defined by a class name and a list of methods.
* Methods have the syntax of regular functions but without the `fun` keyword.
* Fields are not listed in the class definition - they are added to instances dynamically.
* Instances are created by 'calling' a class name like a function - `MyClass(a, b)`.
* Constructors are optional - they are special methods with the name `init`. If a class has no `init` method, its instances are created with no fields (you can add them later).
* Methods can use other class methods or fields of this instance through `this`. 

Example:
```
class Empty {
    print_field() { print this.later; }
}
var e = Empty();
e.later = "hello";
e.print_field(); // hello
print e.later; // hello

class Person {
    init(name) { this.name = name; }
    sayName() { print this.name; }
}

var jane = Person("Jane");
jane.sayName(); // Jane

// classes can be stored in variables
var p = Person;

var bill = p("Bill");
bill.sayName = jane.sayName;
bill.sayName(); // Jane again - functions are first-class and methods bind their instance
```

