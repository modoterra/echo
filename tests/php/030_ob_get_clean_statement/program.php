<?php
// The returned string is ignored, but the active buffer is still removed.
ob_start();
echo "discarded";
ob_get_clean();

// After the buffer is removed, output writes directly to stdout.
echo "after";
