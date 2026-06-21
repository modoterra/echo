<?php
// Angle conversion helpers bridge human-facing degree values and radian-based math.
// Sources:
// - https://www.php.net/manual/en/function.deg2rad.php
// - https://www.php.net/manual/en/function.rad2deg.php
$right_angle = 90;
$negative_angle = -45;

echo "roundtrip-90:[" . intval(rad2deg(deg2rad($right_angle))) . "]\n";
echo "roundtrip-neg45:[" . intval(rad2deg(deg2rad($negative_angle))) . "]\n";
echo "rough-pi:[" . intval(deg2rad(180)) . "]\n";
echo "bool-rad:[" . intval(rad2deg(true)) . "]\n";
echo "string-deg:[" . intval(deg2rad("180")) . "]\n";
echo "exists:[" . function_exists("deg2rad") . function_exists("rad2deg") . "]\n";
