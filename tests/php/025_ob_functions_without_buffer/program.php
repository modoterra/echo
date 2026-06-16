<?php
// The CLI starts with no user-level output buffer.
echo ob_get_level();

// These functions fail without an active user-level output buffer.
ob_flush();
ob_end_flush();
ob_end_clean();
ob_clean();

// Failed calls do not create or remove any user-level output buffer.
echo ob_get_level();
