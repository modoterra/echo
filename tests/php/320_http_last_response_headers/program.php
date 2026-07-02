<?php
echo "before:" . gettype(http_get_last_response_headers()) . "\n";
http_clear_last_response_headers();
echo "after:" . gettype(http_get_last_response_headers()) . "\n";
echo "exists:" . function_exists("http_get_last_response_headers") . function_exists("http_clear_last_response_headers") . "\n";
