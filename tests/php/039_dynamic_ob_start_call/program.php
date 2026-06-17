<?php

// Variable function calls resolve at runtime and must not be lowered as static builtins.
// Source: https://www.php.net/manual/en/functions.variable-functions.php
$fn = "ob_start";
$fn();
echo "buffered";
ob_end_flush();
