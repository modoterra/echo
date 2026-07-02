<?php
echo "depth:" . version_compare("1", "1.0", null) . "\n";
echo "padding:" . version_compare("1.01", "1.1", null) . "\n";
echo "release:" . version_compare("1.0RC1", "1.0", null) . "\n";
echo "patch:" . version_compare("1.0pl1", "1.0.0", null) . "\n";
echo "operator-true:" . version_compare("8.2.0", "8.0.0", ">=") . "\n";
echo "operator-false:" . version_compare("8.2.0", "8.0.0", "<") . "\n";
echo "not-equal:" . version_compare("1.0", "1.0.1", "!=") . "\n";
echo "exists:" . function_exists("version_compare") . "\n";
