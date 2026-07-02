<?php
$parts = parse_url("http://username:password@hostname:9090/path?arg=value#anchor");

echo "scheme:" . $parts["scheme"] . "\n";
echo "host:" . $parts["host"] . "\n";
echo "port:" . $parts["port"] . "\n";
echo "user:" . $parts["user"] . "\n";
echo "pass:" . $parts["pass"] . "\n";
echo "path:" . $parts["path"] . "\n";
echo "query:" . $parts["query"] . "\n";
echo "fragment:" . $parts["fragment"] . "\n";
echo "exists:" . function_exists("parse_url") . "\n";
