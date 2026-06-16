<?php
// The outer buffer captures output until it is explicitly flushed.
ob_start();
echo "outer:";

// The inner buffer captures its own output.
ob_start();
echo "inner";

// Cleaning the inner buffer returns its bytes without flushing them to the parent.
$value = ob_get_clean();

// The outer buffer is still active, so this output is captured there.
echo "|after:";
echo $value;

// Only the outer buffer contents become visible here.
ob_end_flush();
