<?php
clearstatcache();
echo "default cleared\n";

clearstatcache(true, "Cargo.toml");
echo "named cleared\n";

echo "file:[" . file_exists("Cargo.toml") . "]\n";
