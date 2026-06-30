<?php
$token = "Bearer abc123";
$report = "invoice-2026-draft.txt";

echo "token:[" . substr_replace($token, "redacted", 7) . "]\n";
echo "insert:[" . substr_replace($report, "-final", -4, 0) . "]\n";
echo "replace-window:[" . substr_replace("abcdef", "XX", 2, 3) . "]\n";
echo "negative-length:[" . substr_replace("abcdef", "YY", 2, -1) . "]\n";
echo "past-end:[" . substr_replace("abc", "!", 99) . "]\n";
echo "exists:[" . function_exists("substr_replace") . "]\n";
