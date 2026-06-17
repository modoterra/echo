<?php

// strtoupper() and strtolower() convert only ASCII alphabetic bytes.
// Sources:
// https://www.php.net/manual/en/function.strtoupper.php
// https://www.php.net/manual/en/function.strtolower.php
$mixed = "Echo äÖ 123!";
echo strtoupper($mixed) . "\n";
echo strtolower($mixed) . "\n";
echo strtoupper("already UPPER") . "\n";
echo strtolower("already lower") . "\n";
