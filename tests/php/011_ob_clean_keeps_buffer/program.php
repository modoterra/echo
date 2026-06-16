<?php
// Start one output buffer.
ob_start();
// This text will be removed by ob_clean().
echo "discarded";
// Clean clears the active buffer but keeps it active.
ob_clean();
// This text is captured by the same still-active buffer.
echo "kept\n";
// Flush and remove the buffer.
ob_end_flush();
