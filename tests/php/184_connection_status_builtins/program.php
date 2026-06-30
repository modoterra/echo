<?php
echo "aborted: " . connection_aborted() . "\n";
echo "status: " . connection_status() . "\n";

if (connection_status() === 0) {
    echo "normal\n";
}
