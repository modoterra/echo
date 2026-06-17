<?php
// ord() interprets the first byte of a string; str_rot13() rotates ASCII
// letters while leaving non-alpha bytes untouched.
// Sources:
// https://www.php.net/manual/en/function.ord.php
// https://www.php.net/manual/en/function.str-rot13.php
echo ord("A") . "\n";
echo ord("Ä") . "\n";
echo str_rot13("Echo PHP 4.3.0 ÄÖ!") . "\n";
echo str_rot13(str_rot13("Round trip")) . "\n";
