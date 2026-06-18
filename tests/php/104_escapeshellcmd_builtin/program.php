<?php
// escapeshellcmd() escapes shell metacharacters in a command string.
// Source: https://www.php.net/manual/en/function.escapeshellcmd.php
echo "plain:[" . escapeshellcmd("ls -la") . "]\n";
echo "semicolon:[" . escapeshellcmd("path; rm -rf /") . "]\n";
echo "ampersand:[" . escapeshellcmd("foo & bar") . "]\n";
echo "paired-single:[" . escapeshellcmd("echo 'ok'") . "]\n";
echo "unpaired-single:[" . escapeshellcmd("echo 'unterminated") . "]\n";
echo "dollar:[" . escapeshellcmd("a" . chr(36) . "b") . "]\n";
echo "exists:[" . function_exists("escapeshellcmd") . "]\n";
