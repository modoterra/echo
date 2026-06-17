<?php
// strpbrk() returns the tail from the first byte contained in the mask.
// Source: https://www.php.net/manual/en/function.strpbrk.php
echo strpbrk("This is a Simple text.", "mi") . "\n";
echo "missing:" . strpbrk("abcdef", "xy") . "\n";
echo strpbrk("12345", 34) . "\n";
echo "nonascii:" . strpbrk("Ächo", "Ä") . "\n";
