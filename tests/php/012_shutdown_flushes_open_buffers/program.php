<?php
// Start an outer buffer and write one byte into it.
ob_start();
echo "A";
// Start an inner buffer and write one line into it.
ob_start();
echo "B\n";
// No explicit ob_end_flush() calls: PHP shutdown flushes inner then outer.
