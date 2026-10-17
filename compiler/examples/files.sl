module| Files
    fun| string args| | > main > int
        file f("Readme.md" txt wr+)
        string s
        f.open
        'Sail writes from left to right.\n' > f
        f.close
        f.open
        repeat| not f.eof
            f > s
            s '\n' > out
        |repeat
        f.close
        back| 0 |back
    |fun
|module