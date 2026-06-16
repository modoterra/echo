<?php
// The outer buffer captures all output until it is explicitly flushed.
ob_start();
echo "outer:";

// The inner buffer captures its own output first.
ob_start();
echo "inner";

// Flushing the inner buffer sends bytes to the parent buffer, not stdout.
$value = ob_get_flush();

// This output is still captured by the outer buffer.
echo "|after:";
echo $value;

// Only ending the outer buffer makes all captured output visible.
ob_end_flush();
