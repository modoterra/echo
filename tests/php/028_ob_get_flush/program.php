<?php
// Capture output in a user-level buffer.
ob_start();
echo "buffered";

// ob_get_flush() returns the buffered bytes and also flushes them outward.
$value = ob_get_flush();

// The active buffer has been removed, so this writes directly to stdout.
echo "|after:";

// The returned value remains usable after the buffer has been flushed and removed.
echo $value;
