<?php
$constants = get_defined_constants();
echo "has-version:[" . array_key_exists("PHP_VERSION", $constants) . "]\n";
echo "has-sapi:[" . array_key_exists("PHP_SAPI", $constants) . "]\n";
echo "has-password:[" . array_key_exists("PASSWORD_BCRYPT", $constants) . "]\n";
echo "has-missing:[" . array_key_exists("DEFINITELY_MISSING_ECHO_CONSTANT", $constants) . "]\n";
echo "sapi:[" . $constants["PHP_SAPI"] . "]\n";
$categorized = get_defined_constants(true);
echo "has-core:[" . array_key_exists("Core", $categorized) . "]\n";
echo "core-version:[" . array_key_exists("PHP_VERSION", $categorized["Core"]) . "]\n";
echo "exists:[" . function_exists("get_defined_constants") . "]\n";
