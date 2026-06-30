<?php
echo "default-low:[" . round(3.4) . "]\n";
echo "default-half:[" . round(3.5) . "]\n";
echo "default-negative-half:[" . round(-1.5) . "]\n";
echo "precision-positive:[" . round(5.055, 2) . "]\n";
echo "precision-negative:[" . round(678, -2) . "]\n";
echo "string-num:[" . round("12.6") . "]\n";
echo "exists:[" . function_exists("round") . "]\n";
