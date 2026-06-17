<?php
// strrpos() returns the byte position of the last occurrence of the needle.
// Source: https://www.php.net/manual/en/function.strrpos.php
echo strrpos("abcabc", "ab") . "\n";
echo strrpos("abcabc", "bc") . "\n";
echo "missing:" . strrpos("abcdef", "xy") . "\n";
echo strrpos("abcdef", "") . "\n";
echo strrpos("1234545", 45) . "\n";
echo strrpos("Ächocho", "c") . "\n";
