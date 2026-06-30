<?php
echo "positive:[" . intdiv(17, 5) . "]\n";
echo "negative-left:[" . intdiv(-17, 5) . "]\n";
echo "negative-right:[" . intdiv(17, -5) . "]\n";
echo "both-negative:[" . intdiv(-17, -5) . "]\n";
echo "string-numbers:[" . intdiv("42", "6") . "]\n";
echo "exists:[" . function_exists("intdiv") . "]\n";
