<?php
// Start one output buffer.
ob_start();
// Capture this line in the buffer.
echo "flushed\n";
// Flush moves the buffer contents to stdout but leaves the buffer open.
// Script shutdown later flushes the empty still-open buffer.
ob_flush();
