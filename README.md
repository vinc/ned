NED
===


NAME
----

ned - line-oriented text editor


SYNOPSIS
--------

`ned [options] [file]`


DESCRIPTION
-----------

Ned is the new standard text editor.


EXAMPLE
-------

    $ ned
    > a
    Ed is the standard text editor.
    .
    > w example.txt
    > p
    Ed is the standard text editor.
    > %s/standard/venerable/
    > %g/ve.*ble/p
    Ed is the venerable text editor.
    > 1i
    Ned is the new standard text editor.
    .
    > 2d
    > ,p
    Ned is the new standard text editor.
    > w
    > q


SEE ALSO
--------

ed(1), sed(1)


COPYRIGHT
---------

Copyright (c) 2019 Vincent Ollivier. Released under the MIT License.
