<?php
// Capture output in a user-level buffer instead of writing to stdout.
ob_start();
echo "buffered";

// ob_get_clean() returns the buffered bytes and turns the buffer off.
$value = ob_get_clean();

// This writes directly to stdout because the buffer is no longer active.
echo "after:";

// The returned value remains usable after the buffer has been removed.
echo $value;
