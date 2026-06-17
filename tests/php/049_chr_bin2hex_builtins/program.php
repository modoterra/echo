<?php
// chr() generates one byte from an integer; bin2hex() converts string bytes to
// lowercase hexadecimal with the high nibble first.
// Sources:
// https://www.php.net/manual/en/function.chr.php
// https://www.php.net/manual/en/function.bin2hex.php
echo chr(65) . "\n";
echo chr(321) . "\n";
echo bin2hex("Echo") . "\n";
echo bin2hex("Ä") . "\n";
echo bin2hex(chr(195)) . "\n";
