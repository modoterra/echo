<?php
// Start one user-level output buffer.
ob_start();
// This byte is buffered.
echo "x";
// Flush sends buffered bytes outward but keeps the buffer active.
ob_flush();
// This byte is written into the still-active buffer.
echo "y\n";
// Ending with flush emits the remaining buffered bytes.
ob_end_flush();
