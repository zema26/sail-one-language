module| Spatial
    fun| > main > int
        int rows(2) columns(3)
        float weights|2, rows, columns|
        0.0 > weights|*|
        1.5 > weights|0, 2|
        2.5 > weights|1, 0|
        'Matrix: ' weights|0, 2| ' ' weights|1, 0| '\n' > out
        'Elements: ' weights.length '\n' > out

        int volume|3, 2, 2, 2|
        7 > volume|*|
        9 > volume|1, 0, 1|
        'Volume: ' volume|0, 0, 0| ' ' volume|1, 0, 1| '\n' > out

        int original|3|
        4 > original|*|
        'Vector: ' original|2| '\n' > out
        back| 0 |back
    |fun
|module
