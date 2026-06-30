<?php
echo php_sapi_name() . "\n";
if (php_sapi_name() === PHP_SAPI) {
    echo "matches\n";
} else {
    echo "different\n";
}
