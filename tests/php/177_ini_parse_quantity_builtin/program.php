<?php
echo "m:" . ini_parse_quantity("256M") . "\n";
echo "g:" . ini_parse_quantity("4G") . "\n";
echo "hex:" . ini_parse_quantity("0x10") . "\n";
echo "bin:" . ini_parse_quantity("0b1010") . "\n";
echo "oct:" . ini_parse_quantity("010") . "\n";
echo "suffix:" . ini_parse_quantity("10F") . "\n";
echo "bad:" . ini_parse_quantity("foobar") . "\n";
