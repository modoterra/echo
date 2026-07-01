<?php
echo "sapi:[" . constant("PHP_SAPI") . "]\n";
echo "eol:[" . (constant("PHP_EOL") === "\n") . "]\n";
echo "hash:[" . constant("HASH_HMAC") . "]\n";
echo "bcrypt:[" . constant("PASSWORD_BCRYPT") . "]\n";
echo "defined:[" . defined("PHP_SAPI") . "]\n";
echo "exists:[" . function_exists("constant") . "]\n";
