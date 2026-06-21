<?php

putenv("ECHO_CONFIG_MODE=staging");
echo "env:" . getenv("ECHO_CONFIG_MODE") . "\n";

putenv("ECHO_CONFIG_MODE=");
echo "env-empty:[" . getenv("ECHO_CONFIG_MODE") . "]\n";

putenv("ECHO_CONFIG_MODE");
echo "env-cleared:[" . is_bool(getenv("ECHO_CONFIG_MODE")) . "]\n";

$all = getenv();
echo "env-array:[" . is_array($all) . "]\n";

$host = gethostname();
echo "host-string:[" . is_string($host) . "]\n";

$pid = getmypid();
echo "pid-int:[" . is_int($pid) . "]\n";
