<?php
// dechex() returns the unsigned hexadecimal string representation of an integer.
// Source: https://www.php.net/manual/en/function.dechex.php
echo "ten:[" . dechex(10) . "]\n";
echo "forty-seven:[" . dechex(47) . "]\n";
echo "zero:[" . dechex(0) . "]\n";
echo "negative:[" . dechex(-1) . "]\n";
echo "numeric-string:[" . dechex("255") . "]\n";
echo "exists:[" . function_exists("dechex") . "]\n";
