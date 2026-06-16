<?php
// Start one output buffer.
ob_start();
// The active buffer now contains A.
echo "A";
// ob_get_contents() returns a copy of the current buffer contents.
$value = ob_get_contents();
// Mutating the buffer after the copy must not change $value.
echo "B";
// Discard the active buffer so only the copied value can reach stdout.
ob_end_clean();
// The copied value is still A, not AB.
echo $value, "\n";
