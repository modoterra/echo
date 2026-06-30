<?php
if (ini_alter("memory_limit", "128M") === false) {
    echo "memory_limit false\n";
}

if (ini_alter("include_path", ".:/app") === false) {
    echo "include_path false\n";
}
