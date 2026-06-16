<?php
// Start the outer output buffer.
ob_start();
// This byte stays in the outer buffer.
echo "A";
// Start an inner output buffer.
ob_start();
// This byte stays in the inner buffer until it is flushed outward.
echo "B";
// Ending the inner buffer with flush appends its bytes to the parent buffer.
ob_end_flush();
// This byte is written to the outer buffer after the inner buffer is gone.
echo "C\n";
// Ending the outer buffer with flush writes the accumulated output to stdout.
ob_end_flush();
