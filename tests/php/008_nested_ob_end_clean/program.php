<?php
// Start the outer output buffer.
ob_start();
// This byte stays in the outer buffer.
echo "A";
// Start an inner output buffer.
ob_start();
// This inner output should be discarded only from the inner buffer.
echo "discarded";
// Cleaning the inner buffer must not affect the outer buffer.
ob_end_clean();
// This byte is written to the still-active outer buffer.
echo "C\n";
// Flush the outer buffer to stdout.
ob_end_flush();
