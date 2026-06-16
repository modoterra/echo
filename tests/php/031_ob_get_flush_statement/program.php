<?php
// The returned string is ignored, but the active buffer is still flushed and removed.
ob_start();
echo "flushed";
ob_get_flush();

// After the buffer is removed, output writes directly to stdout.
echo "|after";
