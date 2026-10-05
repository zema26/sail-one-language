# Sail Language 

Completely new OOP language with new **classes** and style **elegance**

Welcome to the description of the **Sail programming language**. This manual will walk you through Sail's unique syntax, program structure, and features, using enclosed examples

**Sail** is designed with a primary goal: to make **AI coding** easier, much like natural writing. This is achieved through a few core syntactical changes that streamline the AI development process. 

It has unifying rule that assignments, variable and function declarations and function calls, and input / output always flow from left to right

[Sail Compiler](https://sailcompiler.replit.app)

\---

## 1\. The Basics: Syntax \& Variables

First, let's cover the fundamental building blocks of the Sail language.

### Key Concepts

**It's time to change syntax of programming languages**

* **First: New Assignment Operator:** The most significant change is the assignment operator. Instead of `variable = value`, Sail uses `value > variable`. This allows for a natural left-to-right flow mimicking handwriting, for example: `a + b > a`. 
It's time to break up from Assembly time where operator was on the left  and operands on the right: `MOV R1, R2`

But in conditionals symbol `>`  'greater than' stays the same

* **Second: Renamed Keywords:** 
  * if - when
  * else - other
  * while - repeat
  * for - iterate
  * function - fun 
  * return - back

* **Third: Semitagged Keywords:** Control flow and structural keywords are started and ended with symbol "|", much like a markup language. Examples include `repeat|`...`|repeat` , `fun|`...`|fun` , and `when|`...`|when` This makes it easy to find mistakes. So, blocks are explicit and self-closing with pipes. 

* **Comments:** Single-line comments begin with `//`.

### Data Types & Operators

Sail supports standard data types:

* `int`: integer
* `float`: float decimal
* `char`: character
* `string`: vector of "char"
* `bool`: boolean "true" or "false"

It also supports standard arithmetic operators: `+`, `-`, `*`, `/`, and `%` (mod) .

### Variable Declaration

You can declare a variable with or without an initial value

* **Declaration only:** `int i`
* **Declaration with initialization:** `int i(0)` (i becomes 0)   , `float(3.14)` , `char c("a")`   , `bool l(true)`   , `string s("abcde")` .
* **Declaration of several variables** of same type are separated by whitespace ` int a b ` 

\---

## 2\. Program Structure \& Functions

Sail programs are organized into modules and functions.

### Modules

A program is defined within a `module|` block.

```Sail
module| Euclidean 		// module itself
    ...
|module
```

### Function Declaration

Function declarations are also reversed to fit the left-to-right flow .

* **Syntax:** `fun|   <input_parameters> > <function_name> > <return_type>`
* **Example:** `fun| int a int b > Euclid > int`   (This declares a function named `Euclid` that takes two `int` parameters and returns an `int` ). Parameters are separated by whitespace
* **Calling a function:** `a b > f`

### The `main` Function

The entry point for your program is the `main` function.

* **Declaration:** `fun|  string args| | > main > int`

### Example 1: `Euclidean` Program

This full example demonstrates a simple module, function declaration, and Sail's powerful I/O syntax.

```Sail
module| Euclidean 		// module itself

    fun| int a int b > Euclid > int  // function with type int
							
	repeat| b!= 0
		when| a > b
			    a - b > a
		    other|
			    b - a > b
		    |other
    	|when
	|repeat

	back| a |back     //this return of value by function, not a vector

    |fun

  
  fun|	string args| | > main > int	// main function

	int a b

    in > a  b > Euclid > out

    // combined input and output and function call
  // seamless input to output syntax

  |fun

|module
```

Notice the line: `in > a  b > Euclid > out`. This single line seamlessly:

1. Takes input (`in`)
2. Assigns it to variables `a` and `b`
3. Passes `a` and `b` to the `Euclid` function
4. Takes the return value from `Euclid`
5. Sends that result to output (`out`)

Here, in control flow of program: input, function call and output are seamed together in one transparent fluid motion, allowing coding in the direction from left to right , like in regular handwriting

\---

## 3\. Control Flow & Vectors

Sail provides standard control flow mechanisms and vector support.

### Conditional: `when|` / `other|`

The `Euclidean` example shows a simple `when|` / `other|` block.

```Sail
when| a > b
        a - b > a
    other|
        b - a > b
    |other
|when
```

### Neuro-Symbolic Branching

Standard symbolic logic (`when| / other|`) requires absolute booleans. 
An AI-optimized language benefits from heuristic evaluation operators that handle floating-point probabilities natively, allowing the language to compile generative behaviors.
 * Syntax: `infer| / fall|`
 * Fuzzy Operator (`~`): Evaluates proximity or confidence thresholds.

```Sail 
float confidence(0.85)

infer| confidence ~ 0.90
    // Executes if the confidence tensor meets the acceptable threshold
    "High Probability" > out
fall|
    // Acts as a fallback for sub-threshold evaluations
    "Recalculating" > out
|infer
```

### Loop: `repeat|`

The `Euclidean` example also uses a `repeat|` loop.

```Sail
repeat| b!= 0
    ...
|repeat
```

### Vectors

### Multidimensional Spatial Primitives

AI processing environments rely on high-dimensional vectors rather than linear arrays. While the `| |` vector brackets are distinct, they should natively support tensor shapes to handle multi-layered data arrays without requiring nested loops.
 * **Tensor** Declaration: `type name|rank, shape...|`
 * 1D Vector (Original):`int primes|n|`
 * 2D Matrix: `float weights|2, 64, 64|` (Declares a rank-2 tensor, 64x64)
 * Broadcast Assignment: `0.0 > weights|*|` (Fills all elements simultaneously, eliminating the need for iterate| loops for simple population tasks).

* We use vectors instead of arrays because they are more efficient
* Notice | | brackets instead of [ ]

  * **Declaration:** `int a|n|` (declares an integer vector with `n` elements).
  * **Initialization:** `int a|4|(|1, 2, 3, 4|)`.
  * **Access:** `a|i|` (0-based indexing).

### Loop: `iterate|`

Sail has a specific syntax for `iterate|` loops .

* **Syntax:** `iterate|int i(0)++ <n`
* **Explanation:**

  * `int i(0)`: The iterator `i` is declared and initialized to 0 .
  * `++`: The iteration step is by 1 .
  * `< n`: The loop continues as long as `i` is less than `n` .

Here is an example of a `iterate|` loop used to populate an vector:

```Sail
iterate| int k(0)++ < n       // fill with true
    true > primes|k|
|iterate
```

\---

## 4\. Advanced Example 2: Sieve of Eratosthenes

The following program, `primes`, uses vectors and loops to implement the Sieve of Eratosthenes algorithm.

```Sail
module| primes 

        fun| int n > Eratosthenes > int       // function declaration
             
            bool primes | n |        // variable declaration
            int l(0) i(0) index_square(3)         

            int first last factor 

            iterate| int k(0)++ < n       // fill with true
                true > primes|k|
            |iterate   
                
            repeat| index_square < n         
                when| primes|i|             

                    0 + index_square > first
                    0 + n > last  
                    i + i + 3 > factor
                    false > primes|first|  

                    repeat| last - first > factor 
                        first + factor > first
                        false > primes|first|   
                    |repeat
                    
                |when  
                i + 1 > i 
                2 * i * (i + 3) + 3 > index_square   

            |repeat          

            ' 2' >  out  // print out

            iterate| i(0)++ < n         // print out
                when| primes|i| 
                    when| 2 * i + 3 > n 
                         break
                    |when

                    ' ' 2 * i + 3  >  out
                    l + 1 > l
 
                    when| l % 10 == 0 
                        '\n'  >  out
                    |when   
          
                |when        // when
            |iterate         // print out

            '\n number: ',  l  >  out

        |fun            // erato fun
     
        fun|  string args| | > main > int 		// main function
 
                   1000 > Eratosthenes       // function call

        |fun

|module
```

This example demonstrates vector manipulation (e.g., `false > primes|first|`), nested loops (`repeat|` inside `repeat|`  ), and sending formatted output to the console (e.g., `'\\n' > out` ).

\---



## 5\. Intermodular

Sail modules are connected through Intermodulars.

### Intermodular

Every module can be connected to other modules through intermodulars. Intermodular goes before module itself

### Example: Intermodular and Module

This example demonstrates flexibility and interconnection of module using mentioned before `Euclidian` algorithm

```Sail

intermodular|

   link| in2out.sln |link  //using standard input and output
   link| module2.sln |link  //connecting to another module

    fun| int a int b > Euclid > int |fun  //function that can be used by another modules

|intermodular

module| Euclidean

    fun| int a int b > Euclid > int

    ...

    |fun

    ...

|module

```

In the beginning intermodular gives module access to standard I/O and then access to functions of another module. Also declaration of function that can be used by another module. Notice that source code extension of Sail modules is `.sl`

\---

## 6\. File System

### File Declaration

`file f(name, type, options)` - file declaration and creating

* name includes path
* type can be: bin, txt, or hex
* options: r - read, w - write

### Example: writing and reading text file

```Sail

file f("Readme.md" txt wr+)  //creating text file
string s //srtring for reading and writing

f.open
repeat| s  
//string for writing exists like reading "in > s" from standard input
    s > f  //writing string to file
|repeat
f.close

f.open
repeat| not f.eof  //reading
    f > s  //reading string from file
|repeat
f.close

```

\---

## 7\. Compilation-Time Semantics

Compilation-Time Type Polymorphism: Uses explicit compile-time variable and operator binding (e.g., &T and &Op parameters) to handle generics via procedural specialization rather than complex template instantiation rules

Sail replaces preprocessors and templates with `comp| ... |comp` blocks, evaluating standard code at build time

```Sail

module| generics

comp|                                                  // compilation-time executing
    fun| &T a &Op &T b > Result > &T       // T - type Op - operator

         back| a Op b |back       // this is function return 

    |fun
|comp

fun| string args| | > main > int

    float a_fl(1.41) b_fl(1.73)       // float variables

    int a_int(1) b_int(2)              //integer ones


    a_fl + b_fl > Result > out          //result is sum of floats and equal 3.14

    a_int + b_int > Result > out    //result is sum of integers and equal 3


    a_fl * b_fl > Result > out          //result is product of floats and equal 2.44

    a_int * b_int > Result > out    //result is product of integers and equal 2

|fun

|module

```

Here you can see usage of generics for **different types**: ` float ` and ` integer ` and **different operators**: ` + ` and ` * `

\---

## 8\. Object-Oriented Programming: Compilation-Time Classes

* **Sail** also supports classes, allowing for object-oriented design.
* **Classes** also can be declared and processed during Compilation-Time

* **Declaration:** `class| ClassName ... |class`.
* **Members:** You can define data members (e.g., `int a, int b`)   and member functions (e.g., `fun| ... Euclid`)  inside a class.
* **Object Creation:** `GCD N` creates an instance (object) `N` of the class `GCD`.
* **Access:** Members are accessed using the dot operator (e.g., `N.a`, `N.Euclid`).

### Example 3: `GCD` Class

Here is the `Euclidean` algorithm refactored into a class.

```Sail
module| Euclidean

 comp|               //class processed at Compilation-Time   

    class| GCD    //class declaration

        int a int b    //data members

        fun| int a int b > Euclid > int    //member function

            repeat| b!=0
               when| a > b
                        a - b > a
                    other|
                        b - a > b
                    |other
                |when
            |repeat

            back| a |back

        |fun
    |class

    |comp


    fun| string args| | > main > int   //main function

        GCD N    //object N of class GCD

         in > N.a N.b > N.Euclid > out    
         
         //input, output in one line

    |fun

|module
```

Once again, the `main` function showcases Sail's natural flow.  The line `in > N.a N.b > N.Euclid > out` reads input directly into the object's data members `N.a`, `N.b`, calls the object's member function `N.Euclid`, and prints the returned result.

\---

## 9\. Concurrency

**Sail** achieves concurrency through **Multitex**:

```Sail

Multitex tasks||       //vector of tasks

a b > function1 > tasks|0|     //list of fuctions for each task
i n > function2 > tasks|1|
...

int i(0)
repeat| !eot        //end of time of execution

tasks|i++|.concurr      //running tasks concurrently

|repeat

```

\---

## 10\. Memory Management

Sail has built-in garbage collector

\---

## Origins

* **Sail** language has its roots in **C++** and **Common Lisp** programming languages
* It has variable declaration ` int i ` from **C++** and blocks that are explicit and self-closing with pipes are from **Common Lisp** multi-line comments:

```Common Lisp
#|
....
....

|#

```

* **Sail** input/output:
```Sail
in > a b > Euclid > out
```

* is inspired by **C++** ` stream ` input/output:
```C++
cin >> a >> b 
cout << Euclid(a, b)
```
<zema58@proton.me>