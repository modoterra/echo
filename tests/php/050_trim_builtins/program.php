<?php
// trim(), ltrim(), and rtrim() strip the default ASCII whitespace byte set
// from the requested edge(s) of a string.
// Sources:
// https://www.php.net/manual/en/function.trim.php
// https://www.php.net/manual/en/function.ltrim.php
// https://www.php.net/manual/en/function.rtrim.php
$spaced = "\t Echo \n";
echo bin2hex(trim($spaced)) . "\n";
echo bin2hex(ltrim($spaced)) . "\n";
echo bin2hex(rtrim($spaced)) . "\n";
echo bin2hex(trim(" Ä ")) . "\n";
