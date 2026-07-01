<?php
echo "exists:[" . function_exists("get_error_handler") . "]\n";
echo "initial-null:[" . is_null(get_error_handler()) . "]\n";
