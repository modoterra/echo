<?php
// Start a user-level output buffer and write three bytes into it.
ob_start();
echo "abc";

// Reading the length does not clear or flush the active buffer.
echo ob_get_length();

// The buffered bytes are still present and flush after the length output.
ob_end_flush();
