<?php
echo wordwrap("The quick brown fox jumps", 10) . "\n";
echo "---\n";
echo wordwrap("The quick brown fox", 12, "|") . "\n";
echo "---\n";
echo wordwrap("abcdefghij", 4) . "\n";
echo "---\n";
echo wordwrap("abcdefghij", 4, "\n", true) . "\n";
