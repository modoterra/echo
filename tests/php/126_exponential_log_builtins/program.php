<?php
// Exponential and logarithmic helpers preserve PHP scalar coercion and
// floating-point edge cases.
// Sources:
// - https://www.php.net/manual/en/function.exp.php
// - https://www.php.net/manual/en/function.expm1.php
// - https://www.php.net/manual/en/function.log.php
// - https://www.php.net/manual/en/function.log10.php
// - https://www.php.net/manual/en/function.log1p.php
// - https://www.php.net/manual/en/function.pow.php
echo "exp0:[" . exp(0) . "]\n";
echo "expm1-zero:[" . expm1(0) . "]\n";
echo "log-base:[" . log(8, 2) . "]\n";
echo "log-zero:[" . log(0) . "]\n";
echo "log10:[" . log10(1000) . "]\n";
echo "log1p-zero:[" . log1p(0) . "]\n";
echo "log1p-domain:[" . is_nan(log1p(-2)) . "]\n";
echo "pow-int:[" . pow(2, 8) . "]\n";
echo "pow-frac:[" . pow(10, -1) . "]\n";
echo "pow-domain:[" . is_nan(pow(-1, 5.5)) . "]\n";
echo "exists:[" . function_exists("exp") . function_exists("expm1") . function_exists("log") . function_exists("log10") . function_exists("log1p") . function_exists("pow") . "]\n";
