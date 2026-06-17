<?php
// addslashes(), stripslashes(), and quotemeta() operate on string bytes.
// Source: https://www.php.net/manual/en/ref.strings.php
echo chr(65) . "\n";
echo bin2hex(addslashes("A" . chr(39) . chr(34) . chr(92) . "B")) . "\n";
echo bin2hex(stripslashes(addslashes("A" . chr(39) . chr(34) . chr(92) . "B"))) . "\n";
echo bin2hex(addslashes(chr(0))) . "\n";
echo bin2hex(stripslashes("\\0")) . "\n";
echo bin2hex(quotemeta(".\\+*?[^](\$)")) . "\n";
