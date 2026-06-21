<?php
// Rounding and magnitude helpers preserve PHP's float return types and scalar coercion.
// Sources:
// - https://www.php.net/manual/en/function.ceil.php
// - https://www.php.net/manual/en/function.floor.php
// - https://www.php.net/manual/en/function.sqrt.php
// - https://www.php.net/manual/en/function.hypot.php
$price = 9.99;
$discount = 3.14;

echo "ceil-price:[" . gettype(ceil($price)) . ":" . ceil($price) . "]\n";
echo "floor-discount:[" . gettype(floor($discount)) . ":" . floor($discount) . "]\n";
echo "floor-negative:[" . floor(-3.14) . "]\n";
echo "ceil-negative-zero:[" . ceil(-0.1) . "]\n";
echo "ceil-string:[" . ceil("12.2") . "]\n";
echo "floor-bool:[" . floor(true) . "]\n";
echo "sqrt-square:[" . gettype(sqrt(9)) . ":" . sqrt(9) . "]\n";
echo "sqrt-approx:[" . intval(sqrt(10) * 1000) . "]\n";
echo "sqrt-negative:[" . gettype(sqrt(-1)) . ":" . is_nan(sqrt(-1)) . "]\n";
echo "hypot-3-4:[" . gettype(hypot(3, 4)) . ":" . hypot(3, 4) . "]\n";
echo "hypot-string:[" . intval(hypot("5", "12") * 1000) . "]\n";
echo "exists:[" . function_exists("ceil") . function_exists("floor") . function_exists("sqrt") . function_exists("hypot") . "]\n";
