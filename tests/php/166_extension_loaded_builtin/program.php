<?php
if (extension_loaded("json")) {
    echo "json loaded\n";
} else {
    echo "json missing\n";
}

if (extension_loaded("JSON")) {
    echo "upper loaded\n";
} else {
    echo "upper missing\n";
}
