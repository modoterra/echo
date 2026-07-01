<?php
$modified = getlastmod();

echo "type:[" . gettype($modified) . "]\n";
echo "exists:[" . function_exists("getlastmod") . "]\n";
