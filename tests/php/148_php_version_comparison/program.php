<?php
if (PHP_VERSION_ID < 50600) {
    echo "old\n";
}

if (!(PHP_VERSION_ID < 50600)) {
    echo "new\n";
}
