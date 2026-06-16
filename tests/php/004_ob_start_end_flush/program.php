<?php
// Start one user-level output buffer.
ob_start();
// Output is captured by the active buffer instead of stdout.
echo "Buffered output\n";
// Ending with flush sends the buffer contents to stdout and removes it.
ob_end_flush();
