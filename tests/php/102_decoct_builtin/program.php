<?php
// decoct() returns the unsigned octal string representation of an integer.
// Source: https://www.php.net/manual/en/function.decoct.php
echo "fifteen:[" . decoct(15) . "]\n";
echo "two-sixty-four:[" . decoct(264) . "]\n";
echo "zero:[" . decoct(0) . "]\n";
echo "negative:[" . decoct(-1) . "]\n";
echo "numeric-string:[" . decoct("255") . "]\n";
echo "exists:[" . function_exists("decoct") . "]\n";
