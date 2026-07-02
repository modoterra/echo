<?php
$opened = openlog("echo-test", 0, 8);
$logged = syslog(6, "echo compatibility log message");
$closed = closelog();

echo "opened:" . $opened . "\n";
echo "logged:" . $logged . "\n";
echo "closed:" . $closed . "\n";
echo "exists:" . function_exists("openlog") . function_exists("syslog") . function_exists("closelog") . "\n";
