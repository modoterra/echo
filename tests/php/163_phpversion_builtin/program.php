<?php
echo phpversion() . "\n";
echo phpversion(null) . "\n";
if (phpversion("json") === false) {
    echo "false\n";
} else {
    echo "extension\n";
}
