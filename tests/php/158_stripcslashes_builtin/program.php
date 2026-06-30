<?php
$escaped = chr(92) . "n" . chr(92) . "t" . chr(92) . "x41" . chr(92) . "101";

echo bin2hex(stripcslashes($escaped)) . "\n";
echo bin2hex(stripcslashes(chr(92) . "0")) . "\n";
echo stripcslashes("Path" . chr(92) . chr(92) . "file") . "\n";
