<?php
// strrev() reverses the input string; ucfirst()/lcfirst() convert only the
// first ASCII alphabetic byte.
// Sources:
// https://www.php.net/manual/en/function.strrev.php
// https://www.php.net/manual/en/function.ucfirst.php
// https://www.php.net/manual/en/function.lcfirst.php
$mixed = "echo ÄÖ 123!";
echo strrev($mixed) . "\n";
echo ucfirst("echo") . "\n";
echo ucfirst("Ächo") . "\n";
echo lcfirst("Echo") . "\n";
echo lcfirst("Ächo") . "\n";
