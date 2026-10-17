module| Euclidean
    fun| int a int b > Euclid > int
        repeat| b != 0
            when| a > b
                a - b > a
            other|
                b - a > b
            |other
            |when
        |repeat
        back| a |back
    |fun

    fun| string args| | > main > int
        int a b
        in > a b > Euclid > out
        '\n' > out
        back| 0 |back
    |fun
|module