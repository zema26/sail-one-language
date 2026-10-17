module| primes
    fun| int n > Eratosthenes > int
        bool primes|n|
        int l(0) i(0) index_square(3)
        int first last factor
        iterate| int k(0)++ < n
            true > primes|k|
        |iterate
        repeat| index_square < n
            when| primes|i|
                index_square > first
                n > last
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
        '2' > out
        iterate| i(0)++ < n
            when| primes|i|
                when| 2 * i + 3 > n
                    break
                |when
                ' ' 2 * i + 3 > out
                l + 1 > l
                when| l % 10 == 0
                    '\n' > out
                |when
            |when
        |iterate
        '\nnumber: ' l > out
        '\n' > out
        back| l |back
    |fun
    fun| string args| | > main > int
        100 > Eratosthenes
        back| 0 |back
    |fun
|module