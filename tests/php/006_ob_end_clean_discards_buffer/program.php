<?php
// Start one user-level output buffer.
ob_start();
// This output should be discarded by ob_end_clean().
echo "discarded";
// Ending with clean removes the buffer without flushing it.
ob_end_clean();
// With no active buffer, this writes directly to stdout.
echo "kept\n";
