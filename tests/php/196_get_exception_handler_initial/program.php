<?php
echo "exists:[" . function_exists("get_exception_handler") . "]\n";
echo "initial-null:[" . is_null(get_exception_handler()) . "]\n";
