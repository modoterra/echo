<?php
// Start the outer output buffer.
ob_start();
// This byte stays in the outer buffer.
echo "A";
// Start an inner output buffer.
ob_start();
// This byte starts in the inner buffer.
echo "B";
// Flushing the inner buffer moves B to the parent but keeps the inner buffer active.
ob_flush();
// This byte is written to the still-active inner buffer.
echo "C";
// Ending the inner buffer with flush moves C to the parent.
ob_end_flush();
// With the inner buffer gone, this byte is written to the outer buffer.
echo "D\n";
// Flush the outer buffer to stdout.
ob_end_flush();
