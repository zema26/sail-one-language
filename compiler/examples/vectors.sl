module| Vectors
    fun| string args| | > main > int
        int numbers|4|(|1, 2, 3, 4|)
        int total(0)
        iterate| int i(0)++ < numbers.length
            total + numbers|i| > total
        |iterate
        'Sum: ' total '\n' > out
        string greeting('Sail')
        'Hello, ' + greeting + '!\n' > out
        back| 0 |back
    |fun
|module