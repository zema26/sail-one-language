module| generics
    comp|
        fun| &T a &Op &T b > Result > &T
            back| a Op b |back
        |fun
    |comp
    fun| string args| | > main > int
        float a_fl(1.41) b_fl(1.73)
        int a_int(1) b_int(2)
        a_fl + b_fl > Result > out
        '\n' > out
        a_int + b_int > Result > out
        '\n' > out
        a_fl * b_fl > Result > out
        '\n' > out
        a_int * b_int > Result > out
        '\n' > out
        back| 0 |back
    |fun
|module