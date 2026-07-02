<?php
echo "star:" . fnmatch("*.php", "index.php") . "\n";
echo "question:" . fnmatch("file?.txt", "file1.txt") . "\n";
echo "miss:" . fnmatch("*.php", "index.txt") . "\n";
echo "exists:" . function_exists("fnmatch") . "\n";
