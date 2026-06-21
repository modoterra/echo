<?php
// str_pad() pads byte strings to a target length and truncates the pad
// string when the remaining width is not a multiple of the pad string.
// Source: https://www.php.net/manual/en/function.str-pad.php
echo "right:[" . str_pad("ID", 6, "0") . "]\n";
echo "left:[" . str_pad("42", 5, "0", 0) . "]\n";
echo "both:[" . str_pad("tag", 8, "-", 2) . "]\n";
echo "multi-left:[" . str_pad("42", 7, "ab", 0) . "]\n";
echo "multi-both:[" . str_pad("go", 7, "ab", 2) . "]\n";
echo "shorter:[" . str_pad("already", 3, "0", 0) . "]\n";
echo "negative:[" . str_pad("ok", -2, "0") . "]\n";
echo "empty-input:[" . str_pad("", 3, "xy") . "]\n";
echo "coerce:[" . str_pad(12, "5", 0, "0") . "]\n";
echo "exists:[" . function_exists("str_pad") . "]\n";
