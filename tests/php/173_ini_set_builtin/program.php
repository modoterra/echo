<?php
if (ini_set("memory_limit", "128M") === false) {
    echo "memory_limit false\n";
}

if (ini_set("include_path", ".:/app") === false) {
    echo "include_path false\n";
}
