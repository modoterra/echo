<?php
// is_object() returns true only for object values.
// Source: https://www.php.net/manual/en/function.is-object.php
echo "int:[" . is_object(42) . "]\n";
echo "bool:[" . is_object(true) . "]\n";
echo "null:[" . is_object(null) . "]\n";
echo "string:[" . is_object("value") . "]\n";
echo "array:[" . is_object([1, 2]) . "]\n";
