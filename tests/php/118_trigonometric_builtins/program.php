<?php
// Trigonometric helpers operate on radians and convert back to degrees at boundaries.
// Sources:
// - https://www.php.net/manual/en/function.sin.php
// - https://www.php.net/manual/en/function.cos.php
// - https://www.php.net/manual/en/function.tan.php
// - https://www.php.net/manual/en/function.asin.php
// - https://www.php.net/manual/en/function.acos.php
// - https://www.php.net/manual/en/function.atan.php
// - https://www.php.net/manual/en/function.atan2.php
$thirty = deg2rad(30);
$sixty = deg2rad(60);
$forty_five = deg2rad(45);

echo "sin30:[" . intval(sin($thirty) * 1000 + 0.5) . "]\n";
echo "cos60:[" . intval(cos($sixty) * 1000 + 0.5) . "]\n";
echo "tan45:[" . intval(tan($forty_five) * 1000 + 0.5) . "]\n";
echo "asin-half:[" . intval(rad2deg(asin(0.5)) + 0.5) . "]\n";
echo "acos-half:[" . intval(rad2deg(acos(0.5)) + 0.5) . "]\n";
echo "atan-one:[" . intval(rad2deg(atan(1)) + 0.5) . "]\n";
echo "atan2-nw:[" . intval(rad2deg(atan2(3, -3)) + 0.5) . "]\n";
echo "string:[" . intval(sin("0.5") * 1000 + 0.5) . "]\n";
echo "bool:[" . intval(cos(true) * 1000 + 0.5) . "]\n";
echo "nan-domain:[" . gettype(acos(2)) . ":" . is_nan(acos(2)) . "]\n";
echo "exists:[" . function_exists("sin") . function_exists("cos") . function_exists("tan") . function_exists("asin") . function_exists("acos") . function_exists("atan") . function_exists("atan2") . "]\n";
