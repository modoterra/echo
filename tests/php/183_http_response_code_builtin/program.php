<?php
if (http_response_code() === false) {
    echo "unset\n";
}

if (http_response_code(201) === true) {
    echo "first code set\n";
}

echo "current: " . http_response_code() . "\n";
echo "previous: " . http_response_code(404) . "\n";
echo "current: " . http_response_code() . "\n";
