<?php
// Comment markers inside strings are literal text, not comments.
echo "// not a comment\n";
// A hash inside a string is also literal text.
echo "# not a comment\n";
// Block comment delimiters inside a string are also literal text.
echo "/* not a comment */\n";
