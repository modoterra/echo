<?php
// With no active user-level output buffer, PHP returns false.
echo "before:";

// Echoing false produces an empty string.
echo ob_get_length();

echo ":after";
