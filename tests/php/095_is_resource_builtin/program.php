<?php
// is_resource() detects active PHP resource values.
// Source: https://www.php.net/manual/en/function.is-resource.php
echo "int:[" . is_resource(42) . "]\n";
echo "bool:[" . is_resource(true) . "]\n";
echo "null:[" . is_resource(null) . "]\n";
echo "string:[" . is_resource("value") . "]\n";
echo "array:[" . is_resource([1, 2]) . "]\n";
