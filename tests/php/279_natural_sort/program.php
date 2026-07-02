<?php
$natural = ["img12.png", "img10.png", "img2.png", "img1.png"];
natsort($natural);
echo "natsort:" . implode("|", $natural) . "\n";

$case = ["Img12.png", "img10.png", "img2.png", "Img1.png"];
natcasesort($case);
echo "natcasesort:" . implode("|", $case) . "\n";
echo "natsort-exists:" . function_exists("natsort") . "\n";
echo "natcasesort-exists:" . function_exists("natcasesort") . "\n";
