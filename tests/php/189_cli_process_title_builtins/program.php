<?php
if (cli_get_process_title() === null) {
    echo "unset\n";
}

if (cli_set_process_title("echo worker")) {
    echo "set\n";
}

echo "title:" . cli_get_process_title() . "\n";
