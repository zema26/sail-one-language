module| Euclidean
    comp|
        class| GCD
            int a int b
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
        |class
    |comp
    fun| string args| | > main > int
        GCD N
        in > N.a N.b > N.Euclid > out
        '\n' > out
        back| 0 |back
    |fun
|module