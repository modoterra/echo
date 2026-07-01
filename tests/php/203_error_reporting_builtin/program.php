<?php
echo "initial:[" . error_reporting() . "]\n";
echo "set0:[" . error_reporting(0) . "]\n";
echo "after0:[" . error_reporting() . "]\n";
echo "set-null:[" . error_reporting(null) . "]\n";
echo "after-null:[" . error_reporting() . "]\n";
echo "set5:[" . error_reporting(5) . "]\n";
echo "after5:[" . error_reporting() . "]\n";
echo "exists:[" . function_exists("error_reporting") . "]\n";
