<?php
if (get_extension_funcs("json") === false) {
    echo "json false\n";
}

if (get_extension_funcs("JSON") === false) {
    echo "upper false\n";
}
