<?php
// These functions return false with no active user-level output buffer.
echo "contents:";
echo ob_get_contents();

// Echoing false produces an empty string.
echo ":clean:";
echo ob_get_clean();

// ob_get_flush() also fails with no active buffer; diagnostic parity is deferred.
echo ":flush:";
echo ob_get_flush();

echo ":done";
