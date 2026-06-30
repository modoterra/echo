<?php
header("X-Test: one");
echo "header queued\n";

header("HTTP/1.1 404 Not Found", true, 404);
echo "status header queued\n";
