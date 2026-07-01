<?php
echo "version:[" . defined("PHP_VERSION") . "]\n";
echo "version-id:[" . defined("PHP_VERSION_ID") . "]\n";
echo "sapi:[" . defined("PHP_SAPI") . "]\n";
echo "eol:[" . defined("PHP_EOL") . "]\n";
echo "build-date:[" . defined("PHP_BUILD_DATE") . "]\n";
echo "case:[" . defined("php_version") . "]\n";
echo "missing:[" . defined("DEFINITELY_MISSING_ECHO_CONSTANT") . "]\n";
echo "exists:[" . function_exists("defined") . "]\n";
